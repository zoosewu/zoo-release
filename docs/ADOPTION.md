# 接入 zoo-release（給各專案）

zoo-release 是 zoosewu 各專案共用的發布流程、更新機制與 UI 規範。
這份文件寫給**要接入的專案**（以及在那個專案工作的 AI 助手）。**不要修改 zoo-release repo**；發現規範有問題時在回報中提出。

**目前的版本：v1**

## 步驟

各階段可以分開進行、分開 commit。

### A. 發布流程

1. **改用慣例式提交**：之後的 commit 照 [RELEASE.md](RELEASE.md) 第 1 節的格式。不用改寫舊的 commit。
2. **改寫 `.github/workflows/release.yml`**，照 RELEASE.md 第 4 節：
   - 原生 Rust：保留專案自己的建置與測試，打包好的檔案用 `actions/upload-artifact` 上傳成 `release-*`，發布交給 `zoosewu/zoo-release/.github/workflows/release.yml@v1`。
     **移除**專案自己建立 Release、產生 sha256、產生 Release 說明的步驟。
   - Tauri：建置改用 `tauri-build.yml@v1`，發布用 `release.yml@v1`。
   - Docker：映像改用 `docker-image.yml@v1`，再呼叫 `release.yml@v1`（不帶 `platforms`）。
3. **版本號以 tag 為準**：建置時把 tag 的版本寫進專案設定檔。專案原本就有這種腳本的話沿用。
4. **Release 說明的固定文字**（安裝方式等）放到 `.github/release-notes.md`。
5. 專案原本有 `cliff.toml` 的話可以保留，沒有就用 zoo-release 的預設。
6. **測試**：在 `release` job 加上 `publish: false`，推一個測試 tag（例如 `v0.0.0-test.1`），確認草稿 Release 的內容後刪掉草稿與 tag，再移除 `publish: false`。

簽章用的 secrets 由使用者在 zoo-release 統一設定，專案不用處理。缺少 secrets 時 `release.yml` 會失敗並說明原因。

### B. 更新機制與 UI

1. 照 [UPDATE.md](UPDATE.md) 實作檢查、提示、下載與安裝：
   - 原生 Rust：使用 `zoo-update`：
     ```toml
     zoo-update = { git = "https://github.com/zoosewu/zoo-release", tag = "v1.0.0" }
     ```
     專案原本的更新程式碼改成呼叫它，HTTP 與畫面保留專案自己的實作。
   - Tauri：`tauri-plugin-updater` 的 `endpoints` 與 `pubkey` 照 UPDATE.md，提示規則在前端實作。
   - Docker：伺服器端檢查，管理頁顯示提示與更新指令。
2. 照 [UI.md](UI.md) 實作主畫面橫幅與「關於」頁，用詞照第 3 節的表。
3. 「關於」頁的「作者的其他專案」照 apex-code 的 API 指引實作。

### C. 授權

照 [LICENSING.md](LICENSING.md)：產品用 AGPL-3.0（`AGPL-3.0-only`）＋商業授權，函式庫與工具用 MIT。

1. `LICENSE` 放對應的全文；套件設定檔的 `license` 填 SPDX。
2. 產品的 README 加上「授權」段落（含商業授權的說明）。
3. **貢獻授權**（LICENSING.md 第 5 節）：所有發 PR 的人都必須同意。
   - `CONTRIBUTING.md` 放貢獻授權條款（已有 CONTRIBUTING.md 的話加一節）。
   - `.github/pull_request_template.md` 加上同意勾選行（已有 PR 範本的話加一行）。
   - `.github/workflows/contributor-license.yml` 引用 `contributor-license.yml@v1`。
   - 回報時提醒使用者到 repo 設定把 `contributor-license / agreement` 設為必要檢查。
4. 授權和原本不同時（例如原本是 MIT），**先回報使用者，不要自行更改**：改授權需要確認所有貢獻者的同意。

### D. 在本專案的 CLAUDE.md（或 AGENTS.md）加入這段

**原文照貼，不要翻譯或改寫**，特別是標題那行：zoo-release 靠 `zoo-release: v1` 這個標記確認本專案依照哪一版。

```markdown
## 發布、更新與授權（zoo-release: v1）

發布流程、更新機制、相關 UI 與授權依 https://github.com/zoosewu/zoo-release 的規範：
- commit 使用慣例式提交；推 tag `vX.Y.Z` 發布（docs/RELEASE.md）
- 更新檢查、提示與安裝（docs/UPDATE.md），新版本橫幅與「關於」頁（docs/UI.md）
- 授權依 docs/LICENSING.md：產品 AGPL-3.0-only＋商業授權、函式庫與工具 MIT；加入 GPL／AGPL 的相依套件前先問使用者
- 外部 PR 必須勾選同意 CONTRIBUTING.md 的貢獻授權才能合併；不要合併沒有同意的 PR
- 修改發布、更新或「關於」頁前先讀對應的規範，不要自行偏離；規範不適用時回報使用者
- 使用者說「更新 zoo-release 規範」時：讀 docs/ADOPTION.md 的「變更紀錄」，只做從目前版本到最新版本列出的「專案要做的事」，最後把上面標題的版本號改成最新版
```

### E. 回報

- 完成了哪些階段（A、B、C、D）
- 測試 Release 的結果
- 規範不適用或需要使用者決定的部分

## 版本原則

- 版本號**只有在各專案必須做事時才會增加**。文字修正、補充說明、zoo-release 自己能處理的變更都不升版。
- 可重用 workflow 用 `@v1` 引用：同一個主版號內只做相容的變更，專案不用改就會用到新版。
- 各專案只照「變更紀錄」裡的「專案要做的事」調整，不需要重讀整份規範。

## 變更紀錄

### v1（2026-11-27）

首版。專案要做的事：上面的 A、B、C、D、E。
