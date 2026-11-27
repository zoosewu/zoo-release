use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Version;
use crate::verify::{self, Error};

/// `latest.json` 的固定網址（GitHub 官方文件記載的「最新 Release 檔案」網址，不經過 API）
pub fn manifest_url(repo: &str) -> String {
    format!("https://github.com/{repo}/releases/latest/download/latest.json")
}

/// Release 頁面：`version` 為 None 時是 Release 清單
pub fn release_page_url(repo: &str, version: Option<Version>) -> String {
    match version {
        Some(v) => format!("https://github.com/{repo}/releases/tag/v{v}"),
        None => format!("https://github.com/{repo}/releases"),
    }
}

/// 目前平台在 `latest.json` 的鍵（`windows-x86_64` 等）
pub fn current_platform() -> &'static str {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86") {
        "i686"
    } else if cfg!(target_arch = "arm") {
        "armv7"
    } else {
        "x86_64"
    };
    // 組合都是編譯期決定的，列舉出來以回傳 'static 字串
    match (os, arch) {
        ("windows", "aarch64") => "windows-aarch64",
        ("windows", "i686") => "windows-i686",
        ("windows", _) => "windows-x86_64",
        ("darwin", "aarch64") => "darwin-aarch64",
        ("darwin", _) => "darwin-x86_64",
        (_, "aarch64") => "linux-aarch64",
        (_, "armv7") => "linux-armv7",
        (_, "i686") => "linux-i686",
        _ => "linux-x86_64",
    }
}

/// `latest.json`（相容 Tauri updater，加上 `sha256`）。不認得的欄位略過。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub pub_date: Option<String>,
    #[serde(default)]
    pub platforms: BTreeMap<String, PlatformAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformAsset {
    pub url: String,
    /// `.sig` 檔的內容（Tauri／minisign 格式，base64）
    pub signature: String,
    #[serde(default)]
    pub sha256: Option<String>,
}

impl Manifest {
    pub fn parse(json: &str) -> Result<Self, Error> {
        serde_json::from_str(json).map_err(|e| Error::Manifest(e.to_string()))
    }

    /// 解析失敗或是預先發行版時為 None（視為沒有新版）
    pub fn version(&self) -> Option<Version> {
        Version::parse(&self.version)
    }

    pub fn asset(&self, platform: &str) -> Option<&PlatformAsset> {
        self.platforms.get(platform)
    }
}

impl PlatformAsset {
    /// 只接受 `https://github.com/<repo>/releases/download/` 開頭的網址
    pub fn check_url(&self, repo: &str) -> Result<(), Error> {
        let prefix = format!("https://github.com/{repo}/releases/download/");
        let rest = self.url.strip_prefix(&prefix).ok_or_else(|| Error::Url(self.url.clone()))?;
        // <tag>/<檔名>，不能再往上跳
        let ok = rest.split('/').count() == 2 && !rest.contains("..") && !rest.contains(['?', '#', '\\']);
        if ok { Ok(()) } else { Err(Error::Url(self.url.clone())) }
    }

    /// 驗證下載的內容：簽章（必須）＋ sha256（有提供時）
    pub fn verify(&self, data: &[u8], public_keys: &[&str]) -> Result<(), Error> {
        verify::verify_signature(data, &self.signature, public_keys)?;
        self.verify_sha256(&verify::sha256_hex(data))
    }

    /// 驗證已下載的檔案（逐段讀取，不必整個載入記憶體）
    pub fn verify_file(&self, path: &std::path::Path, public_keys: &[&str]) -> Result<(), Error> {
        let sha = verify::verify_signature_file(path, &self.signature, public_keys)?;
        self.verify_sha256(&sha)
    }

    fn verify_sha256(&self, actual: &str) -> Result<(), Error> {
        match &self.sha256 {
            Some(expected) if !expected.eq_ignore_ascii_case(actual) => Err(Error::Sha256),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZOO_DOWNLOADER: &str = include_str!("../tests/fixtures/zoo-downloader-0.6.0-latest.json");

    #[test]
    fn urls() {
        assert_eq!(manifest_url("zoosewu/z-overlay"), "https://github.com/zoosewu/z-overlay/releases/latest/download/latest.json");
        assert_eq!(release_page_url("zoosewu/x", Some(Version::new(0, 7, 0))), "https://github.com/zoosewu/x/releases/tag/v0.7.0");
        assert_eq!(release_page_url("zoosewu/x", None), "https://github.com/zoosewu/x/releases");
        let p = current_platform();
        assert!(["windows", "darwin", "linux"].iter().any(|os| p.starts_with(os)), "{p}");
    }

    #[test]
    fn parse_tauri_manifest() {
        let m = Manifest::parse(ZOO_DOWNLOADER).unwrap();
        assert_eq!(m.version(), Some(Version::new(0, 6, 0)));
        let win = m.asset("windows-x86_64").unwrap();
        assert!(win.url.ends_with("zoo-downloader_0.6.0_x64-setup.exe"));
        assert!(win.sha256.is_none()); // Tauri 產生的沒有 sha256
        win.check_url("zoosewu/zoo-downloader").unwrap();
    }

    #[test]
    fn parse_minimal_and_extra_fields() {
        let m = Manifest::parse(r#"{"version":"1.2.3","future":{"x":1}}"#).unwrap();
        assert_eq!(m.version(), Some(Version::new(1, 2, 3)));
        assert!(m.platforms.is_empty());
        assert_eq!(Manifest::parse(r#"{"version":"1.2.3-rc.1"}"#).unwrap().version(), None);
        assert!(Manifest::parse("not json").is_err());
        assert!(Manifest::parse(r#"{"notes":"no version"}"#).is_err());
    }

    #[test]
    fn check_url_rejects_other_places() {
        let asset = |url: &str| PlatformAsset { url: url.into(), signature: String::new(), sha256: None };
        let repo = "zoosewu/z-overlay";
        asset("https://github.com/zoosewu/z-overlay/releases/download/v0.2.0/z-overlay.exe").check_url(repo).unwrap();
        for bad in [
            "http://github.com/zoosewu/z-overlay/releases/download/v0.2.0/z-overlay.exe",
            "https://github.com/zoosewu/other/releases/download/v0.2.0/z-overlay.exe",
            "https://github.com/zoosewu/z-overlay/releases/download/v0.2.0/../x.exe",
            "https://github.com/zoosewu/z-overlay/releases/download/v0.2.0/a/b.exe",
            "https://github.com/zoosewu/z-overlay/releases/download/v0.2.0/x.exe?y",
            "https://evil.example/zoosewu/z-overlay/releases/download/v0.2.0/x.exe",
        ] {
            assert!(asset(bad).check_url(repo).is_err(), "{bad}");
        }
    }
}
