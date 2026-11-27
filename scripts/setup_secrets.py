# /// script
# requires-python = ">=3.9"
# dependencies = ["pynacl"]
# ///
"""把共用的簽章金鑰設定到 scripts/repos.txt 列出的每個 repo（docs/SIGNING.md）。

直接呼叫 GitHub REST API，不需要 gh。設定兩個 secret：
  ZOO_SIGNING_KEY       私鑰檔的內容
  ZOO_SIGNING_PASSWORD  私鑰的密碼
私鑰與密碼的來源（擇一）：
  --keepass   直接讀 KeePassXC 資料庫（keepassxc-cli）：密碼欄＋私鑰附件，只詢問主密碼
  --key-file  私鑰檔，執行時詢問私鑰密碼
GitHub 權限依序用：環境變數 GITHUB_TOKEN → git 已登入的認證（Git Credential Manager）→ 執行時詢問 token。
輸入都不會顯示，也不會出現在指令列或歷史紀錄。

用 uv 執行（自動準備 PyNaCl，不會裝進系統的 Python）：

  uv run scripts/setup_secrets.py --keepass D:/path/zoo.kdbx
  uv run scripts/setup_secrets.py --key-file D:/path/zoo.key --repo zoosewu/z-overlay
"""

import argparse
import base64
import getpass
import json
import os
import shutil
import subprocess
import sys
import urllib.error
import urllib.request

API = "https://api.github.com"


def load_repos(path):
    with open(path, encoding="utf-8") as f:
        return [line.split()[0] for line in f if line.strip() and not line.startswith("#")]


def request(method, url, token, body=None):
    req = urllib.request.Request(
        url,
        method=method,
        data=None if body is None else json.dumps(body).encode(),
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "zoo-release-setup-secrets",
        },
    )
    with urllib.request.urlopen(req) as res:
        data = res.read()
        return json.loads(data) if data else None


def git_credential_token():
    """從 git 的認證管理員（Git Credential Manager）取得已登入的 GitHub token；沒有就回傳 None。"""
    try:
        r = subprocess.run(
            ["git", "credential", "fill"],
            input="protocol=https\nhost=github.com\n\n",
            capture_output=True,
            text=True,
            env={**os.environ, "GIT_TERMINAL_PROMPT": "0"},
            timeout=300,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    for line in r.stdout.splitlines():
        if line.startswith("password="):
            return line[len("password="):] or None
    return None


def keepass_read(db, entry, attachment, master, key_file=None, cli="keepassxc-cli"):
    """用 keepassxc-cli 讀出（私鑰附件內容, 密碼欄）。主密碼從標準輸入傳入，不出現在指令列。"""

    def run(*args):
        cmd = [cli, args[0], *(["-k", key_file] if key_file else []), *args[1:]]
        r = subprocess.run(cmd, input=(master + "\n").encode(), capture_output=True)
        if r.returncode != 0:
            msg = r.stderr.decode(errors="replace").strip() or "主密碼錯誤，或資料庫無法開啟"
            sys.exit(f"讀取 KeePassXC 失敗：{msg}")
        return r.stdout.decode("utf-8")

    key = run("attachment-export", "-q", "--stdout", db, entry, attachment).strip()
    password = run("show", "-q", "-s", "-a", "Password", db, entry)
    # 只去掉 CLI 加上的換行，密碼本身的空白保留
    if password.endswith("\n"):
        password = password[:-1]
    if password.endswith("\r"):
        password = password[:-1]
    return key, password


def encrypt(public_key, value):
    from nacl.public import PublicKey, SealedBox

    box = SealedBox(PublicKey(base64.b64decode(public_key)))
    return base64.b64encode(box.encrypt(value.encode())).decode()


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    src = p.add_mutually_exclusive_group(required=True)
    src.add_argument("--keepass", metavar="KDBX", help="KeePassXC 資料庫（.kdbx）")
    src.add_argument("--key-file", help="私鑰檔")
    p.add_argument("--keepass-key-file", metavar="KEYX", help="KeePassXC 資料庫的金鑰檔（有設定才需要）")
    p.add_argument("--entry", default="zoo-release 簽章金鑰", help="KeePassXC 的項目，預設「%(default)s」")
    p.add_argument("--attachment", default="zoo-downloader.key", help="項目裡的私鑰附件，預設 %(default)s")
    p.add_argument("--repo", help="只設定這個 repo（owner/name）；省略時設定 repos.txt 的全部")
    args = p.parse_args()

    try:
        import nacl  # noqa: F401
    except ImportError:
        sys.exit("缺少 PyNaCl：改用 uv run scripts/setup_secrets.py 執行")

    path = args.keepass or args.key_file
    if not os.path.isfile(path):
        sys.exit(f"找不到檔案：{os.path.abspath(path)}")
    if args.keepass:
        if not shutil.which("keepassxc-cli"):
            sys.exit("找不到 keepassxc-cli：把 KeePassXC 的安裝資料夾加進 PATH")
        master = getpass.getpass("KeePassXC 主密碼：")
        key, password = keepass_read(args.keepass, args.entry, args.attachment, master, args.keepass_key_file)
    else:
        with open(args.key_file, encoding="utf-8") as f:
            key = f.read().strip()
        password = getpass.getpass("私鑰密碼：")
    if not key:
        sys.exit("私鑰是空的")

    repos = [args.repo] if args.repo else load_repos(os.path.join(os.path.dirname(__file__), "repos.txt"))
    token = os.environ.get("GITHUB_TOKEN") or git_credential_token()
    if token:
        print("使用這台電腦已登入的 GitHub 權限")
    else:
        token = getpass.getpass("GitHub token（Secrets 讀寫權限）：").strip()

    secrets = {"ZOO_SIGNING_KEY": key, "ZOO_SIGNING_PASSWORD": password}
    failed = []
    for repo in repos:
        try:
            pk = request("GET", f"{API}/repos/{repo}/actions/secrets/public-key", token)
            for name, value in secrets.items():
                request(
                    "PUT",
                    f"{API}/repos/{repo}/actions/secrets/{name}",
                    token,
                    {"encrypted_value": encrypt(pk["key"], value), "key_id": pk["key_id"]},
                )
            print(f"完成 {repo}")
        except urllib.error.HTTPError as e:
            hint = {401: "token 無效或過期", 403: "token 沒有這個 repo 的 Secrets 寫入權限", 404: "找不到 repo，或 token 沒有這個 repo 的權限"}
            print(f"失敗 {repo}：HTTP {e.code} {hint.get(e.code, '')}", file=sys.stderr)
            failed.append(repo)

    print(f"{len(repos) - len(failed)}/{len(repos)} 個 repo 完成")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
