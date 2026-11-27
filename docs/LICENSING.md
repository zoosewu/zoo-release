# 授權

專案沒有特別指定時，一律照這份規定。要例外的專案在自己的 CLAUDE.md 寫明。

## 1. 選哪一種

| 類型 | 授權 | SPDX | 例子 |
|---|---|---|---|
| **產品**：使用者直接安裝、執行或自架的軟體 | **AGPL-3.0 ＋ 商業授權**（雙授權） | `AGPL-3.0-only` | 桌面 app、輸入法、Docker 服務、瀏覽器腳本與擴充 |
| **函式庫與工具**：給其他專案或開發者使用的程式碼 | **MIT** | `MIT` | Rust crate、npm 套件、CLI 開發工具、GitHub Actions、zoo-release |

- 判斷不了時問一句：「別人會把它放進自己的程式嗎？」會 → 函式庫（MIT）；不會，是拿來用的 → 產品（AGPL）。
- 產品 repo 裡附帶的函式庫或開發工具（例如給 VS Code 用的擴充）可以單獨用 MIT，在那個子資料夾放自己的 LICENSE，套件設定檔寫 `MIT`。
- fork 的專案照原專案的授權，不適用這份規定。

### 為什麼函式庫用 MIT

AGPL 會傳染：函式庫用 AGPL，用到它的每個程式都必須以 AGPL 相容的條款發布。著作權人自己不受限，但只要函式庫收過別人的貢獻，產品要提供商業授權時就得取得每位貢獻者同意。用 MIT 就沒有這個問題，AGPL 產品與它的商業授權版本都能直接使用。

## 2. 產品要放的東西

| 位置 | 內容 |
|---|---|
| `LICENSE` | AGPL-3.0 全文（<https://www.gnu.org/licenses/agpl-3.0.txt>），不要改內容 |
| 套件設定檔 | `Cargo.toml`：`license = "AGPL-3.0-only"`；`package.json`：`"license": "AGPL-3.0-only"`；workspace 用 `license.workspace = true` |
| README | 下面的「授權」段落 |

### README 的授權段落

```markdown
## 授權

本專案以 [AGPL-3.0](LICENSE) 授權。如需以 AGPL-3.0 以外的條款使用（例如閉源的商業產品），可另外取得商業授權，請透過 [GitHub Issues](https://github.com/zoosewu/<repo>/issues) 聯絡。
```

英文 README：

```markdown
## License

Licensed under [AGPL-3.0](LICENSE). A commercial license is available for use outside the terms of the AGPL-3.0 (for example in closed-source products); please get in touch through [GitHub Issues](https://github.com/zoosewu/<repo>/issues).
```

商業授權的條件與價格另外討論，不寫在 README。

## 3. 函式庫與工具要放的東西

| 位置 | 內容 |
|---|---|
| `LICENSE` | MIT 全文，著作權人寫 `zoosewu` |
| 套件設定檔 | `license = "MIT"`／`"license": "MIT"` |

## 4. 為了保留商業授權要注意的事

雙授權的前提是**產品的著作權全部屬於你**，所以：

| 項目 | 規定 |
|---|---|
| 外部貢獻（PR） | 合併前取得貢獻者同意，讓你能以商業授權再授權那部分。CONTRIBUTING.md 放下面的聲明，PR 範本加一個勾選項 |
| 第三方相依套件 | 優先用 MIT、Apache-2.0、BSD、ISC、Zlib 等寬鬆授權。**GPL、AGPL 的相依套件會讓商業授權版本無法發布**，加入前先問使用者 |
| 複製別人的程式碼 | 只複製寬鬆授權的，保留原本的著作權聲明 |

CONTRIBUTING.md 的聲明（非正式的法律文件，正式提供商業授權前建議請律師確認）：

```markdown
## 授權

提交貢獻即表示你同意：你的貢獻以本專案的授權（AGPL-3.0）釋出，並同意專案作者另以其他條款（包括商業授權）提供包含你的貢獻的版本。
```

## 5. 檢查

`tools/audit.py` 的「E 授權」檢查產品 repo：`LICENSE` 是 AGPL-3.0、根目錄的套件設定檔是 `AGPL-3.0-only`、README 提到商業授權。
