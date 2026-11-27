"""檢查各專案接入 zoo-release 的狀態（docs/ADOPTION.md 的 A、B、C、D）。只用標準函式庫。

用法：
  python tools/audit.py            讀本機（scripts/repos.txt 的路徑，相對 zoo-release 的上層資料夾）
  python tools/audit.py --remote   讀 GitHub 預設分支（確認已 push）
"""
import json
import re
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LATEST = int(re.search(r"\*\*目前的版本：v(\d+)\*\*", (ROOT / "docs/ADOPTION.md").read_text(encoding="utf-8"))[1])
MARKER = re.compile(r"zoo-release:\s*v(\d+)")
CURRENT_KEY = re.search(r'pub const CURRENT: &str = "([^"]+)"', (ROOT / "crates/zoo-update/src/keys.rs").read_text(encoding="utf-8"))[1]
USES = "zoosewu/zoo-release/.github/workflows/{}@v"
BUILD_WORKFLOW = {"tauri": "tauri-build.yml", "docker": "docker-image.yml"}


def repos():
    for line in (ROOT / "scripts/repos.txt").read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.startswith("#"):
            repo, local, kind = line.split()[:3]
            yield repo, (None if local == "-" else ROOT.parent / local), kind


def fetch(url: str) -> bytes | None:
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent": "zoo-release-audit"}), timeout=20) as r:
            return r.read()
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return None
        raise


class Local:
    def __init__(self, path: Path):
        self.path = path
        self.where = str(path)

    def files(self) -> list[str]:
        out = subprocess.run(["git", "-C", str(self.path), "ls-files"], capture_output=True, text=True, encoding="utf-8")
        return out.stdout.splitlines()

    def read(self, rel: str) -> str | None:
        f = self.path / rel
        return f.read_text(encoding="utf-8", errors="replace") if f.is_file() else None


class Remote:
    def __init__(self, repo: str):
        self.repo = repo
        info = json.loads(fetch(f"https://api.github.com/repos/{repo}") or b"{}")
        self.branch = info.get("default_branch", "main")
        self.where = f"github:{repo}@{self.branch}"

    def files(self) -> list[str]:
        data = fetch(f"https://api.github.com/repos/{self.repo}/git/trees/{self.branch}?recursive=1")
        return [i["path"] for i in json.loads(data)["tree"] if i["type"] == "blob"] if data else []

    def read(self, rel: str) -> str | None:
        data = fetch(f"https://raw.githubusercontent.com/{self.repo}/{self.branch}/{rel}")
        return data.decode("utf-8", "replace") if data is not None else None


def audit(src, kind: str) -> dict[str, list[str]]:
    files = src.files()
    result = {"A 發布": [], "B 更新": [], "C 授權": [], "D 標記": []}

    # A：release workflow 使用共用的 workflow
    workflows = "\n".join(src.read(f) or "" for f in files if re.match(r"\.github/workflows/[^/]+\.ya?ml$", f))
    if USES.format("release.yml") not in workflows:
        result["A 發布"].append("沒有使用 release.yml@v1")
    if kind in BUILD_WORKFLOW and USES.format(BUILD_WORKFLOW[kind]) not in workflows:
        result["A 發布"].append(f"沒有使用 {BUILD_WORKFLOW[kind]}@v1")

    # B：更新機制
    if kind == "rust":
        cargo = "\n".join(src.read(f) or "" for f in files if f.endswith("Cargo.toml"))
        if not re.search(r"^zoo-update\s*=", cargo, re.M):
            result["B 更新"].append("Cargo.toml 沒有 zoo-update 相依")
    elif kind == "tauri":
        # tauri.conf.json 與建置時用 --config 合併的 tauri.*.json
        text = "\n".join(src.read(f) or "" for f in files if re.search(r"(^|/)tauri(\.[\w-]+)?\.conf\.json$|(^|/)tauri\.[\w-]+\.json$", f))
        if "/releases/latest/download/latest.json" not in text:
            result["B 更新"].append("tauri 設定的 updater endpoints 不是 latest.json 網址")
        if CURRENT_KEY not in text:
            result["B 更新"].append("tauri 設定的 pubkey 不是共用公鑰")
        if not re.search(r'"createUpdaterArtifacts"\s*:\s*true', text):
            result["B 更新"].append("tauri 設定沒有 bundle.createUpdaterArtifacts: true")
    elif kind == "docker":
        code = [f for f in files if f.endswith(".go")]
        if not any("releases/latest/download/latest.json" in (src.read(f) or "") for f in code):
            result["B 更新"].append("Go 程式沒有讀取 latest.json")

    # C：授權（repos.txt 列的都是產品：AGPL-3.0-only＋商業授權，docs/LICENSING.md）
    license_text = src.read("LICENSE") or ""
    if "GNU AFFERO GENERAL PUBLIC LICENSE" not in license_text or "Version 3" not in license_text:
        result["C 授權"].append("LICENSE 不是 AGPL-3.0 全文")
    for f in manifests(files):
        m = re.search(r'^\s*license\s*=\s*"([^"]+)"|"license"\s*:\s*"([^"]+)"', src.read(f) or "", re.M)
        if m and (m[1] or m[2]) != "AGPL-3.0-only":
            result["C 授權"].append(f"{f} 的 license 是 {m[1] or m[2]}")
    readme = "\n".join(src.read(f) or "" for f in files if re.match(r"README(\.[\w-]+)?\.md$", f))
    if not re.search(r"商業授權|commercial licen[cs]e", readme, re.I):
        result["C 授權"].append("README 沒有提到商業授權")

    # D：CLAUDE.md／AGENTS.md 的版本標記
    versions = []
    for name in ("CLAUDE.md", "AGENTS.md"):
        versions += [int(v) for v in MARKER.findall(src.read(name) or "")]
    if not versions:
        result["D 標記"].append("CLAUDE.md / AGENTS.md 沒有 zoo-release 標記")
    elif max(versions) < LATEST:
        result["D 標記"].append(f"規範版本 v{max(versions)}，最新為 v{LATEST}")
    return result


# 產品本身的套件設定檔：深度 2 以內，不含附帶的工具、範例、測試與相依套件
# （附帶的函式庫與工具可以用 MIT，docs/LICENSING.md 第 1 節）
EXCLUDED = re.compile(r"(^|/)(tools|examples?|tests?|e2e|fixtures|node_modules|vendor|third_party)/")


def manifests(files: list[str]) -> list[str]:
    return [f for f in files
            if re.search(r"(^|/)(Cargo\.toml|package\.json)$", f) and f.count("/") <= 2 and not EXCLUDED.search(f)]


def main() -> int:
    remote = "--remote" in sys.argv
    print(f"最新規範版本：v{LATEST}\n")
    done = total = 0
    for repo, local, kind in repos():
        total += 1
        src = Local(local) if (local and local.is_dir() and not remote) else Remote(repo)
        try:
            result = audit(src, kind)
        except Exception as e:  # 網路問題不中斷其他專案
            result = {"錯誤": [f"無法檢查：{e}"]}
        ok = not any(result.values())
        done += ok
        print(f"{'✓' if ok else '✗'} {repo} [{kind}]  ({src.where})")
        for part, problems in result.items():
            status = "✓" if not problems else "✗ " + "；".join(problems)
            print(f"    {part}：{status}")
    print(f"\n{done}/{total} 個專案完成")
    return 0 if done == total else 1


if __name__ == "__main__":
    sys.exit(main())
