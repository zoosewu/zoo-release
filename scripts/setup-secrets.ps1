<#
.SYNOPSIS
  把共用的簽章金鑰設定到 scripts/repos.txt 列出的每個 repo（docs/SIGNING.md）。

.DESCRIPTION
  設定兩個 secret：ZOO_SIGNING_KEY（私鑰檔的內容）與 ZOO_SIGNING_PASSWORD。
  需要 GitHub CLI（gh）並已登入。密碼在執行時詢問，不會出現在指令列或歷史紀錄。

.EXAMPLE
  .\scripts\setup-secrets.ps1 -KeyFile "$HOME\.zoo-release\zoo.key"
  .\scripts\setup-secrets.ps1 -KeyFile "$HOME\.zoo-release\zoo.key" -Repo zoosewu/z-overlay
#>
param(
  [Parameter(Mandatory)] [string] $KeyFile,
  # 只設定這個 repo；省略時設定 repos.txt 的全部
  [string] $Repo
)
$ErrorActionPreference = 'Stop'

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { throw '找不到 gh（GitHub CLI）：https://cli.github.com/' }
gh auth status *> $null
if ($LASTEXITCODE -ne 0) { throw 'gh 尚未登入：執行 gh auth login' }
if (-not (Test-Path $KeyFile)) { throw "找不到私鑰檔 $KeyFile" }

$repos = if ($Repo) { @($Repo) } else {
  Get-Content (Join-Path $PSScriptRoot 'repos.txt') |
    Where-Object { $_ -and -not $_.StartsWith('#') } |
    ForEach-Object { ($_ -split '\s+')[0] }
}

$key = (Get-Content $KeyFile -Raw).Trim()
$secure = Read-Host '私鑰密碼' -AsSecureString
$password = [Runtime.InteropServices.Marshal]::PtrToStringBSTR([Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure))

try {
  foreach ($r in $repos) {
    Write-Host "設定 $r"
    # 從標準輸入傳值，避免出現在處理序的命令列
    $key | gh secret set ZOO_SIGNING_KEY --repo $r
    if ($LASTEXITCODE -ne 0) { throw "$r：設定 ZOO_SIGNING_KEY 失敗" }
    $password | gh secret set ZOO_SIGNING_PASSWORD --repo $r
    if ($LASTEXITCODE -ne 0) { throw "$r：設定 ZOO_SIGNING_PASSWORD 失敗" }
  }
  Write-Host "完成：$($repos.Count) 個 repo"
} finally {
  $password = $null
  $key = $null
}
