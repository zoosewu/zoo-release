# 簽章金鑰

所有專案共用**一組** Tauri／minisign 格式的金鑰。Tauri app 的 updater 外掛直接使用；原生 Rust app 用 `zoo-update` 驗證。

## 它能防什麼

| 能防 | 不能防 |
|---|---|
| Release 的檔案被替換、下載時被竄改、下載來源出問題 | GitHub 帳號被盜（拿到帳號就能跑 workflow 用金鑰簽章） |

sha256 和檔案放在同一個 Release，只能確認下載完整，防不了竄改；簽章才行。

## 1. 金鑰

**沿用 zoo-downloader 原有的金鑰**（金鑰 ID `6D1CE0BB9F067C60`）。公鑰已在 zoo-downloader 的 `tauri.conf.json` 與
`crates/zoo-update/src/keys.rs`，zoo-downloader 不用改。

之後需要新金鑰時（換金鑰、遺失）才產生（需要 Node.js）：

```powershell
npx --yes @tauri-apps/cli@2 signer generate -w "$HOME\.zoo-release\zoo.key"
# 會詢問密碼；產生 zoo.key（私鑰）與 zoo.key.pub（公鑰）
```

## 2. 保存：KeePassXC

私鑰與密碼存在 [KeePassXC](https://keepassxc.org/)（開源、原生程式、本機加密檔，不需要雲端帳號）。

| 項目 | 放哪裡 |
|---|---|
| 私鑰 | KeePassXC 的項目「zoo-release 簽章金鑰」：私鑰檔加為**附加檔案**，同時把內容貼在**受保護的附加屬性** `private_key`（方便複製，畫面上隱藏；不要貼在備註，備註會明文顯示） |
| 密碼 | 同一個項目的密碼欄 |
| 公鑰 | 公開。同一個項目的備註，並寫在 `crates/zoo-update/src/keys.rs` 與各 Tauri app 的設定 |
| GitHub | 各 repo 的 secret `ZOO_SIGNING_KEY`、`ZOO_SIGNING_PASSWORD`（設定後讀不回來） |

**KeePassXC 資料庫是私鑰唯一能取回的地方**，所以：

- 主密碼用 4–5 個隨機詞組成的密碼片語。
- 至少兩份備份：資料庫檔（`.kdbx`）放雲端硬碟（OneDrive、Google Drive）同步，另外複製一份到隨身碟，換金鑰或改主密碼後更新。
- 私鑰檔放進 KeePassXC 後，從本機刪除。設定 repo 時從 KeePassXC 匯出附加檔案到暫存資料夾，用完刪掉。

## 3. 設定到各 repo

個人帳號沒有跨 repo 共用的 secret，所以用腳本透過 GitHub REST API 一次設定（不需要 gh）。

**準備**（第一次）：

1. 安裝 [uv](https://docs.astral.sh/uv/)。腳本需要的 PyNaCl（GitHub 要求 secret 先用 libsodium 加密）由 `uv run` 自動準備，不會裝進系統的 Python。
2. GitHub 權限：腳本先用這台電腦 git 已登入的認證（Git Credential Manager）；沒有的話才詢問 token。需要自己建立 token 時：GitHub → Settings → Developer settings → Fine-grained tokens → Generate new token
   - Repository access：Only select repositories，選 `scripts/repos.txt` 列出的 repo
   - Permissions：**Secrets → Read and write**
   - 期限設短（例如 7 天）；token 也存進 KeePassXC，或用完就刪。

**設定**：

```powershell
# 直接讀 KeePassXC（需要 keepassxc-cli 在 PATH）：只詢問主密碼，私鑰取自項目附件、密碼取自密碼欄
uv run scripts\setup_secrets.py --keepass <資料庫.kdbx>
# 資料庫有金鑰檔時：--keepass-key-file <金鑰檔.keyx>
# 項目或附件名稱不同時：--entry "<項目>" --attachment <附件檔名>
# 只設定一個 repo：--repo zoosewu/z-overlay
# 沒有已登入的 GitHub 認證時會再詢問 token；輸入都不會顯示
```

也可以在網頁手動設定：repo 的 Settings → Secrets and variables → Actions → New repository secret。

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
