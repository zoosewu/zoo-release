# zoo-release — 計畫

zoosewu 各專案共用的發布流程、更新機制與 UI 規範（GitHub：`zoosewu/zoo-release`）。

## 決策（2026-11-27）
- 方案 C：獨立 repo，各專案以 `@v1` 引用可重用 workflow 與共用套件
- 發布：推 tag `vX.Y.Z` 觸發建置；慣例式提交 + git-cliff 自動產生 Release 說明
- 更新檢查：讀 `releases/latest/download/latest.json`（GitHub 官方文件的固定網址，無 API 額度），格式相容 Tauri updater
- 提示：只有主／次版號較新才在主畫面提示；可略過某個 主.次 版；只檢查不下載，使用者按了才下載
- 驗證：全部簽章（共用一組 Tauri／minisign 金鑰）＋ sha256
- 範圍：桌面程式（zing、z-overlay、zoo-type、zoo-downloader、waypoint）＋ Docker（zkaraver、kara-creator，只提示不自動更新）
- ani-gamer-history：更新交給 Tampermonkey，發布流程維持 release-please（不在本 repo 範圍）
- UI：只統一結構、行為、用詞（主畫面橫幅、關於頁、用詞表），不統一視覺；「其他專案」放關於頁

## 內容
- [x] docs/RELEASE.md、UPDATE.md、UI.md、SIGNING.md、ADOPTION.md
- [x] cliff.toml（預設的 Release 說明格式）
- [x] scripts/manifest.py（產生 latest.json）＋ 測試
- [x] .github/workflows/release.yml（共用：sha256、簽章、latest.json、說明、建立 Release）
- [x] .github/workflows/tauri-build.yml、docker-image.yml
- [x] crates/zoo-update（Rust：latest.json、版本比較、提示規則、簽章與 sha256 驗證）＋ 測試
- [x] scripts/setup-secrets.ps1（把簽章金鑰設到各 repo）
- [x] tools/audit.py（各專案接入狀態）
- [x] CLAUDE.md、README.md
- [x] 驗證：cargo test、py 測試、workflow YAML 語法

## 待辦
- [x] 授權：zoo-release 與 zoo-update 用 MIT；docs/LICENSING.md（產品 AGPL-3.0-only＋商業授權、函式庫與工具 MIT）；ADOPTION 加入 C 授權；audit 檢查
- [ ] waypoint 沒有 LICENSE、package.json 是 MIT：改 AGPL 前要使用者確認原作者（I321I）的程式碼權利
- [x] 沿用 zoo-downloader 的金鑰；保存在 KeePassXC（docs/SIGNING.md）
- [ ] 執行 scripts/setup-secrets.ps1
- [ ] push、建立 v1.0.0 與 v1 tag
- [ ] 第一個接入的專案用 publish: false 實測 release.yml（z-overlay）
- [x] 所有 PR 都要同意貢獻授權（contributor-license.yml）

## 限制
- 可重用 workflow 無法在本機執行，第一個接入的專案用 `publish: false`（只建草稿）實測
