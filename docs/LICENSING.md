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
| 外部貢獻（PR） | **所有發 PR 的人都必須同意貢獻授權**（第 5 節），沒有同意的 PR 不合併 |
| 第三方相依套件 | 優先用 MIT、Apache-2.0、BSD、ISC、Zlib 等寬鬆授權。**GPL、AGPL 的相依套件會讓商業授權版本無法發布**，加入前先問使用者 |
| 複製別人的程式碼 | 只複製寬鬆授權的，保留原本的著作權聲明 |

## 5. 貢獻授權（所有 PR 都要同意）

發 PR 的人必須同意：**貢獻以 AGPL-3.0 授權**，並且**同意作者另以其他條款（包括商業授權）提供含這份貢獻的版本**。
函式庫與工具（MIT）的 repo 只需要同意以 MIT 授權。

以下文字不是正式法律文件，正式提供商業授權前建議請律師確認。

### 5.1 CONTRIBUTING.md

產品：

```markdown
## 貢獻授權 / Contribution license

發 PR 前請閱讀並同意以下條款，並在 PR 說明中勾選同意。沒有同意的 PR 不會被合併。

1. 你的貢獻以本專案的授權 AGPL-3.0（`AGPL-3.0-only`）釋出。
2. 你同意專案作者（zoosewu）另以其他條款提供包含你的貢獻的版本，包括商業授權，且不需另外通知或支付費用。
3. 你有權提交這份貢獻（是你自己寫的，或來源的授權允許以上兩點）。

By opening a pull request you agree that:

1. Your contribution is licensed under this project's license, AGPL-3.0 (`AGPL-3.0-only`).
2. The project author (zoosewu) may also distribute versions that include your contribution under other terms, including a commercial license, without further notice or payment.
3. You have the right to submit the contribution (you wrote it, or its source license allows the above).
```

函式庫與工具：

```markdown
## 貢獻授權 / Contribution license

你的貢獻以本專案的授權 MIT 釋出。 / Your contribution is licensed under this project's license, MIT.
```

### 5.2 PR 範本 `.github/pull_request_template.md`

保留勾選行的文字，其他內容照各專案需要：

```markdown
- [ ] 我已閱讀並同意 CONTRIBUTING.md 的貢獻授權 / I have read and agree to the contribution license in CONTRIBUTING.md
```

### 5.3 自動檢查 `.github/workflows/contributor-license.yml`

```yaml
name: contributor-license
on:
  pull_request:
    types: [opened, edited, reopened, synchronize]
jobs:
  agreement:
    uses: zoosewu/zoo-release/.github/workflows/contributor-license.yml@v1
```

- PR 說明裡的勾選行沒有打勾時檢查失敗。repo 擁有者與機器人（Dependabot、release-please 等）的 PR 自動通過。
- 設為必要檢查後，沒有同意就無法合併，見 5.4。
- 勾選紀錄留在 PR 上；合併後不要改動 PR 說明。

### 5.4 設為必要檢查

GitHub 的檢查清單只列出**最近 7 天在這個 repo 跑過**的檢查，所以要先讓檢查跑一次，才選得到。

**1. 觸發檢查（由專案的 agent 做）**：`contributor-license.yml` push 到預設分支後，開一個沒有檔案變更的測試 PR。

```sh
git switch -c chore/contributor-license-check
git commit --allow-empty -m "chore: 觸發貢獻授權檢查"
git push -u origin chore/contributor-license-check
# 用 gh 或 GitHub API 開 PR 到預設分支；作者是 repo 擁有者，檢查會自動通過
```

確認檢查通過後，**關閉 PR（不合併）並刪除分支**。

**2. 設定 ruleset（由使用者做）**：Settings → Rules → Rulesets → New ruleset → New branch ruleset

1. **Ruleset name** 填 `main`，**Enforcement status** 選 **Active**。
2. **Bypass list** 加入 **Repository admin**（Always allow）：必要檢查會擋住所有沒跑過檢查的 commit，包括直接 push 到預設分支；這個檢查只在 PR 上跑，不加例外的話擁有者也不能直接 push。
3. **Target branches** → **Add target** → **Include default branch**。
4. 勾 **Require status checks to pass** → **Add checks**，搜尋 `agreement`，選 **`agreement / agreement`**。
   名稱是「呼叫端的 job 名稱 / reusable workflow 的 job 名稱」，不是 workflow 檔名。
5. 按 **Create**。

超過 7 天才設定、清單裡找不到時，再做一次步驟 1。

## 6. 檢查

產品 repo 要符合：`LICENSE` 是 AGPL-3.0、根目錄的套件設定檔是 `AGPL-3.0-only`、README 提到商業授權、CONTRIBUTING.md 有貢獻授權、PR 範本有勾選行、有使用 `contributor-license.yml`。
分支保護的設定需要到 GitHub 確認。
