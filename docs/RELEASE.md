# 發布流程

所有專案用同一套流程：**慣例式提交 → 推 tag → CI 建置 → 共用 workflow 發布**。

## 1. Commit 格式（慣例式提交）

```
<類型>(<範圍>): <說明>

<內文，可省略>
```

| 類型 | 用途 | Release 說明的分組 |
|---|---|---|
| `feat` | 新功能 | 新功能 |
| `fix` | 修正 | 修正 |
| `perf` | 效能 | 效能 |
| `refactor` | 重構 | 重構 |
| `docs` | 文件 | 文件 |
| `test` `build` `ci` `chore` `style` | 其他 | 其他 |

- 範圍可省略，例如 `fix(update): 下載中斷時保留原檔`。
- 不相容的變更在類型後加 `!`（`feat!: …`），或內文寫 `BREAKING CHANGE: …`。
- **不符合格式的 commit 不會出現在 Release 說明**，但不會擋住發布。
- `chore(release): …` 固定跳過（版本號 commit）。
- 說明的語言照各專案慣例（中文或英文），同一個專案內一致。

## 2. 版本號

- 格式 `v主.次.修補`（例如 `v0.7.0`）。`v0.7.0-rc.1` 這類預先發行版會標成 prerelease，不會成為 latest，已安裝的 app 也不會收到。
- **版本號以 tag 為準**：建置時把 tag 的版本寫進專案（`Cargo.toml`、`package.json`、`tauri.conf.json`），之後可選擇把版本號 commit 回預設分支（`chore(release): vX.Y.Z`）。
- 怎麼選版號：
  - 修補 `0.6.4 → 0.6.5`：只有修正。**已安裝的 app 不會在主畫面提示**，只在「關於」頁顯示。
  - 次版 `0.6.x → 0.7.0`：有新功能或使用者看得到的改變。**會在主畫面提示。**
  - 主版 `0.x → 1.0.0`：不相容的變更（設定格式、資料格式）。

## 3. 發布步驟

```sh
git switch main && git pull          # 預設分支，CI 通過
git tag v0.7.0
git push origin v0.7.0
```

推 tag 之後全部由 CI 完成。專案若有自己的升版腳本（例如 `pnpm release`），腳本最後也只做上面這三件事。

## 4. Workflow 結構

```yaml
# <專案>/.github/workflows/release.yml
name: release
on:
  push:
    tags: ['v*']
permissions:
  contents: write
jobs:
  build:            # 專案自己的建置：測試、編譯、打包，把要發布的檔案上傳成 artifact
    ...
    - uses: actions/upload-artifact@v7
      with:
        name: release-windows      # 名稱以 release- 開頭
        path: dist/*
  release:          # 共用的發布
    needs: build
    uses: zoosewu/zoo-release/.github/workflows/release.yml@v1
    with:
      name: zing-explore
      platforms: '{"windows-x86_64": "zing-explore-windows-x64.zip"}'
    secrets: inherit
```

依專案類型：

| 類型 | build | release |
|---|---|---|
| 原生 Rust（zing、z-overlay、zoo-type） | 專案自己寫（測試、打包方式差很多） | `release.yml` |
| Tauri（zoo-downloader、waypoint） | `tauri-build.yml`（共用） | `release.yml` |
| Docker（zkaraver、kara-creator） | `docker-image.yml`（共用，推到 ghcr.io） | `release.yml`（不帶 `platforms`，只產生版本資訊） |

### `release.yml` 做的事

1. 檢查 tag 格式（`v主.次.修補`，可帶 `-預先發行`）。
2. 下載所有 `release-*` artifact。
3. 每個檔案產生 `<檔名>.sha256`。
4. `platforms` 指定的更新檔沒有 `.sig` 的話，用共用金鑰簽章（見 [SIGNING.md](SIGNING.md)）。
5. 產生 `latest.json`（格式見 [UPDATE.md](UPDATE.md)）。
6. Release 說明：git-cliff 依 `cliff.toml` 產生（專案沒有 `cliff.toml` 就用 zoo-release 的預設：`cliff.toml` 中文、`cliff.en.toml` 英文），後面接 `.github/release-notes.md`（安裝說明等固定文字，可省略）。
7. 先建草稿、上傳全部檔案，最後才發布並標成 latest，**已安裝的 app 不會看到上傳到一半的 Release**。

### `release.yml` 的輸入

| 輸入 | 必填 | 說明 |
|---|---|---|
| `name` | | Release 標題的前綴，標題為 `<name> <版本>`；省略時標題為 tag |
| `platforms` | | 更新檔對照：`{"<平台>": "<檔名或萬用字元>"}`。平台鍵見 UPDATE.md。省略＝沒有可自動更新的檔案（Docker） |
| `notes-footer` | | Release 說明後面接的固定文字檔，預設 `.github/release-notes.md`（不存在就略過） |
| `notes-language` | | `zh-TW`（預設）或 `en`：專案沒有自己的 `cliff.toml` 時，Release 說明的分組標題用哪種語言 |
| `publish` | | `false` 時只建草稿不發布，用來測試，預設 `true` |
| `zoo-release-ref` | | 讀取 zoo-release 腳本與預設 `cliff.toml` 的版本，預設 `v1` |

secrets：`ZOO_SIGNING_KEY`（私鑰檔的內容）、`ZOO_SIGNING_PASSWORD`（用 `secrets: inherit` 傳入）。舊名稱 `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 仍可使用，新名稱優先。

## 5. Release 內容

| 檔案 | 用途 |
|---|---|
| 各平台檔案 | 安裝檔、免安裝執行檔、zip |
| `<檔案>.sha256` | 手動下載的人核對用 |
| `<更新檔>.sig` | 簽章，app 更新時驗證 |
| `latest.json` | 已安裝的 app 從 `releases/latest/download/latest.json` 讀取 |

**檔名建議不含版本**（例如 `z-overlay.exe`），使用者可以用固定網址 `releases/latest/download/<檔名>` 下載最新版。Tauri 產生的安裝檔含版本號，不影響更新（網址寫在 `latest.json`）。

## 6. Docker 專案

- `docker-image.yml` 推映像到 `ghcr.io/zoosewu/<映像>`，標籤：`主.次.修補`、`主.次`、`latest`。
- 同時照樣建 GitHub Release（說明與 `latest.json`），讓網頁介面能提示新版本（見 UPDATE.md「Docker」）。
