"""產生 latest.json（格式見 docs/UPDATE.md，相容 Tauri updater）。只用標準函式庫。

用法：
  python scripts/manifest.py --repo zoosewu/z-overlay --tag v0.2.0 --dist dist \
      --platforms '{"windows-x86_64": "z-overlay.exe"}' --notes-file notes.md --out dist/latest.json

platforms 的值可以是檔名或萬用字元（例如 "*_x64-setup.exe"），在 dist 中必須剛好對到一個檔案，
而且旁邊要有 <檔名>.sig。
"""
import argparse
import datetime
import fnmatch
import hashlib
import json
import re
import sys
from pathlib import Path

TAG = re.compile(r"^v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)$")
PLATFORM = re.compile(r"^(windows|darwin|linux)-(x86_64|aarch64|i686|armv7)$")


class ManifestError(Exception):
    pass


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def find_asset(dist: Path, pattern: str) -> Path:
    names = sorted(p.name for p in dist.iterdir() if p.is_file() and not p.name.endswith((".sig", ".sha256")))
    matches = [n for n in names if fnmatch.fnmatchcase(n, pattern)]
    if len(matches) != 1:
        raise ManifestError(f"{pattern!r} 應對到剛好一個檔案，實際：{matches or '沒有'}（dist：{names}）")
    return dist / matches[0]


def build(repo: str, tag: str, dist: Path, platforms: dict[str, str], notes: str, now: datetime.datetime) -> dict:
    m = TAG.match(tag)
    if not m:
        raise ManifestError(f"tag {tag!r} 的格式需為 v主.次.修補（可帶 -預先發行）")
    out_platforms = {}
    for key, pattern in platforms.items():
        if not PLATFORM.match(key):
            raise ManifestError(f"平台鍵 {key!r} 不符合 <windows|darwin|linux>-<x86_64|aarch64|i686|armv7>")
        asset = find_asset(dist, pattern)
        sig = asset.with_name(asset.name + ".sig")
        if not sig.is_file():
            raise ManifestError(f"{asset.name} 沒有簽章檔 {sig.name}")
        out_platforms[key] = {
            "url": f"https://github.com/{repo}/releases/download/{tag}/{asset.name}",
            "signature": sig.read_text(encoding="utf-8").strip(),
            "sha256": sha256_file(asset),
        }
    return {
        "version": m[1],
        "notes": notes.strip(),
        "pub_date": now.astimezone(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": out_platforms,
    }


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", required=True)
    ap.add_argument("--tag", required=True)
    ap.add_argument("--dist", required=True, type=Path)
    ap.add_argument("--platforms", default="{}")
    ap.add_argument("--notes-file", type=Path)
    ap.add_argument("--out", type=Path)
    ap.add_argument("--print-assets", action="store_true", help="只列出 platforms 對到的檔案（每行一個），供簽章步驟使用")
    args = ap.parse_args(argv)
    try:
        platforms = json.loads(args.platforms or "{}")
        if not isinstance(platforms, dict):
            raise ManifestError("platforms 必須是 JSON 物件")
        if args.print_assets:
            for pattern in platforms.values():
                print(find_asset(args.dist, pattern))
            return 0
        if not args.out:
            raise ManifestError("需要 --out")
        notes = args.notes_file.read_text(encoding="utf-8") if args.notes_file else ""
        manifest = build(args.repo, args.tag, args.dist, platforms, notes, datetime.datetime.now(datetime.timezone.utc))
    except (ManifestError, json.JSONDecodeError) as e:
        print(f"::error::latest.json：{e}", file=sys.stderr)
        return 1
    args.out.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
