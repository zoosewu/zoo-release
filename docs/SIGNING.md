# 簽章金鑰

所有專案共用**一組** Tauri／minisign 格式的金鑰。Tauri app 的 updater 外掛直接使用；原生 Rust app 用 `zoo-update` 驗證。

## 它能防什麼

| 能防 | 不能防 |
|---|---|
| Release 的檔案被替換、下載時被竄改、下載來源出問題 | GitHub 帳號被盜（拿到帳號就能跑 workflow 用金鑰簽章） |

sha256 和檔案放在同一個 Release，只能確認下載完整，防不了竄改；簽章才行。

## 1. 準備金鑰（只做一次）

**zoo-downloader 已經有一組金鑰**（它的 `tauri.conf.json` 裡有公鑰、repo 有 secrets）。如果你還保有那組**私鑰與密碼**，直接沿用：zoo-downloader 不用改，其他專案用同一組。

沒有的話產生新的一組（需要 Node.js）：

```powershell
npx --yes @tauri-apps/cli@2 signer generate -w "$HOME\.zoo-release\zoo.key"
# 會詢問密碼；產生 zoo.key（私鑰）與 zoo.key.pub（公鑰）
```

## 2. 保存

| 項目 | 放哪裡 |
|---|---|
| 私鑰（`zoo.key` 的內容） | **密碼管理器**（主要備份）＋ 各 repo 的 secret `TAURI_SIGNING_PRIVATE_KEY` |
| 密碼 | 密碼管理器 ＋ 各 repo 的 secret `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` |
| 公鑰（`zoo.key.pub` 的內容） | 公開。寫在 `crates/zoo-update/src/keys.rs` 與各 Tauri app 的 `tauri.conf.json` |

私鑰檔放進密碼管理器後，可以從本機刪除。

## 3. 設定到各 repo

個人帳號沒有跨 repo 共用的 secret，所以用腳本一次設定（需要 [GitHub CLI](https://cli.github.com/) 並已登入）：

```powershell
.\scripts\setup-secrets.ps1 -KeyFile "$HOME\.zoo-release\zoo.key"
# 會詢問密碼，設定到 scripts/repos.txt 列出的每個 repo
```

新增專案時把 repo 加進 `scripts/repos.txt` 再執行一次。

## 4. 換金鑰

app 內建兩把公鑰：**目前**與**下一把**，任一把驗證通過即可。

1. 產生新金鑰，把新公鑰放進「下一把」，發布各 app 的新版本。
2. 等多數使用者都更新到這個版本後，把各 repo 的 secret 換成新私鑰。
3. 下一次發布時，「目前」改成新公鑰，「下一把」清空（或放更新一把）。

## 5. 遺失私鑰

舊版 app 無法驗證新版本，使用者必須**手動下載一次**新版。所以私鑰一定要備份到密碼管理器。

處理方式：產生新金鑰 → 新版 app 內建新公鑰 → Release 說明提醒使用者手動下載。

## 6. Windows 程式碼簽章（不在這次範圍）

讓 Windows SmartScreen 不顯示警告的 Authenticode 簽章是另一回事，需要付費憑證或雲端簽章服務。之後要做時另外討論。
