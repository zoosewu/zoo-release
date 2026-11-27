# zoo-release

zoosewu 各專案共用的**發布流程、更新機制與 UI 規範**。

| 文件 | 內容 |
|---|---|
| [docs/RELEASE.md](docs/RELEASE.md) | 慣例式提交、推 tag 發布、共用 workflow |
| [docs/UPDATE.md](docs/UPDATE.md) | `latest.json`、檢查時機、提示規則、下載與驗證 |
| [docs/UI.md](docs/UI.md) | 新版本橫幅、「關於」頁、用詞對照表 |
| [docs/SIGNING.md](docs/SIGNING.md) | 共用簽章金鑰的保存、設定與輪替 |
| [docs/ADOPTION.md](docs/ADOPTION.md) | **各專案的接入步驟**（交給各專案的入口） |

## 內容

| 路徑 | 用途 |
|---|---|
| `.github/workflows/release.yml` | 可重用：sha256、簽章、`latest.json`、Release 說明、建立並發布 Release |
| `.github/workflows/tauri-build.yml` | 可重用：Tauri 各平台建置 |
| `.github/workflows/docker-image.yml` | 可重用：Docker 映像推到 ghcr.io |
| `cliff.toml` | 預設的 Release 說明格式（git-cliff） |
| `crates/zoo-update` | Rust：`latest.json`、版本比較、提示規則、簽章與 sha256 驗證 |
| `scripts/manifest.py` | 產生 `latest.json` |
| `scripts/setup-secrets.ps1` | 把簽章金鑰設定到各 repo |
| `scripts/repos.txt` | 使用 zoo-release 的 repo 清單 |
| `tools/audit.py` | 檢查各專案的接入狀態 |

## 使用

```yaml
jobs:
  build:
    # …專案自己的建置，上傳 release-* artifact
  release:
    needs: build
    uses: zoosewu/zoo-release/.github/workflows/release.yml@v1
    with:
      platforms: '{"windows-x86_64": "z-overlay.exe"}'
    secrets: inherit
```

```toml
zoo-update = { git = "https://github.com/zoosewu/zoo-release", tag = "v1.0.0" }
```

## 開發

```sh
python -m unittest discover -s scripts      # 腳本測試
cd crates/zoo-update && cargo test          # zoo-update 測試
python tools/audit.py                       # 各專案接入狀態
```
