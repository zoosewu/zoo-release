# zoo-release

zoosewu 各專案共用的發布流程、更新機制、UI 與授權規範。本 repo 是工具，授權為 MIT（docs/LICENSING.md）。各專案用 `@v1` 引用這裡的可重用 workflow 與 `zoo-update` crate，
並依 `docs/` 的規範實作。使用的 repo 列在 `private/repos.txt`。

**`private/` 不入庫**（.gitignore）：zoo-release 會給別人使用，維護者自己的腳本與清單（`setup_secrets.py`、`repos.txt`、`audit.py`）都放這裡，不要推上去，也不要在公開文件提到。

## 規則

使用者的要求：**規範盡量相容、避免更新；各專案只做必要的調整，其他都由 zoo-release 負責。**

- **能在 zoo-release 做的就不要推給專案**：例如新增 workflow 的輸入（給預設值）、相容舊格式、在共用 workflow 裡多做一步。
- **相容性**：
  - 可重用 workflow：v1 內只能新增**有預設值**的輸入、不改既有輸入的意義與預設值、不新增必填 secret。
  - `zoo-update`：v1 內只新增 API，不改既有函式的簽名與行為。
  - `latest.json`：只新增欄位；讀取端一律略過不認得的欄位。
  - 做不到相容時開 v2（`@v2`、`tag = "v2.x.y"`），v1 保留。
- **改規範前先判斷專案需不需要做事**：
  - 不需要（文字修正、補充說明、放寬規定、zoo-release 能處理的變更）→ 直接改，不升版，不通知。
  - 需要 → 先問自己能不能改成 zoo-release 處理；真的不行才升版：
    1. `docs/ADOPTION.md` 的「目前的版本」與段落範本的標記加一。
    2. 在「變更紀錄」寫新版本與「專案要做的事」（只列必要動作）。
    3. 已完成的部分不要求重做。
- **升版後一定要提醒使用者**：列出需要更新的專案（`py private/audit.py` 中版本落後的），並給可以直接貼到各專案的一句話：
  「更新 zoo-release 規範（https://github.com/zoosewu/zoo-release/blob/main/docs/ADOPTION.md）」。提醒前確認已 push。
- **使用者說各專案改完後一定要檢查**：`py private/audit.py`（已 push 的話加 `--remote`），逐一回報通過與未通過的專案及原因。
  audit 只檢查設定與引用；UI 與更新行為要另外讀程式碼確認。

## 發布 zoo-release 本身

```sh
git tag v1.0.1 && git push origin v1.0.1     # 固定版本（zoo-update 用 tag 引用）
git tag -f v1 && git push -f origin v1       # 移動主版號 tag（workflow 用 @v1 引用）
```

移動 `v1` 會立刻影響所有專案的下一次發布：先在 `ci` 通過、確認相容後才做，並經使用者同意。

## 驗證

```sh
py -m unittest discover -s scripts
cd crates/zoo-update && cargo clippy --all-targets -- -D warnings && cargo test
# 真實 Tauri 簽章：ZOO_UPDATE_FIXTURE=<含 zoo-downloader 0.6.0 setup.exe 的資料夾> cargo test -- --ignored
```

可重用 workflow 無法在本機執行：改動後在一個專案用 `publish: false` 推測試 tag 驗證，確認草稿內容後刪掉草稿與 tag。

## 簽章金鑰

沿用 zoo-downloader 的金鑰（使用者 2026-11-27 確認保有私鑰與密碼），保存在 KeePassXC（docs/SIGNING.md）。
私鑰與密碼只由使用者處理：需要設定 secrets 時請使用者執行 `uv run private/setup_secrets.py`，不要經手私鑰或密碼。
