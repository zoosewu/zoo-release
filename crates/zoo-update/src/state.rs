use serde::{Deserialize, Serialize};

use crate::{Manifest, Version};

/// 兩次自動檢查至少相隔的秒數
pub const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

/// 目前版本與最新版本的關係
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// 相同或較舊，或最新版本無效
    UpToDate,
    /// 只有修補版較新：只在「關於」頁顯示
    Patch(Version),
    /// 主版或次版較新：主畫面也提示（除非略過）
    Feature(Version),
}

impl Availability {
    pub fn version(self) -> Option<Version> {
        match self {
            Availability::UpToDate => None,
            Availability::Patch(v) | Availability::Feature(v) => Some(v),
        }
    }
}

pub fn availability(current: Version, latest: Option<Version>) -> Availability {
    match latest {
        Some(v) if v.feature_line() > current.feature_line() => Availability::Feature(v),
        Some(v) if v > current => Availability::Patch(v),
        _ => Availability::UpToDate,
    }
}

/// 要保存在設定檔的更新狀態（docs/UPDATE.md 第 6 節）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateState {
    /// 上次成功檢查的時間（Unix 秒）
    pub last_check: Option<u64>,
    /// 上次看到的最新版本（`主.次.修補`）
    pub latest: Option<String>,
    /// 略過的 `主.次`（例如 `"0.7"`）
    pub skipped: Option<String>,
    pub auto_check: bool,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self { last_check: None, latest: None, skipped: None, auto_check: true }
    }
}

impl UpdateState {
    /// 自動檢查的時候到了沒有（關閉自動檢查時永遠 false；手動檢查不看這個）
    pub fn check_due(&self, now: u64) -> bool {
        self.auto_check
            && match self.last_check {
                None => true,
                // 時鐘被往回調時也重新檢查
                Some(t) => now < t || now - t >= CHECK_INTERVAL_SECS,
            }
    }

    /// 檢查成功後記下結果
    pub fn record_check(&mut self, now: u64, manifest: &Manifest) {
        self.last_check = Some(now);
        self.latest = manifest.version().map(|v| v.to_string());
    }

    pub fn latest_version(&self) -> Option<Version> {
        self.latest.as_deref().and_then(Version::parse)
    }

    pub fn availability(&self, current: Version) -> Availability {
        availability(current, self.latest_version())
    }

    /// 「略過此版本」：記下 `主.次`，之後同一個 `主.次` 都不在主畫面提示
    pub fn skip(&mut self, version: Version) {
        self.skipped = Some(format!("{}.{}", version.major, version.minor));
    }

    fn skipped_line(&self) -> Option<(u64, u64)> {
        let (major, minor) = self.skipped.as_deref()?.split_once('.')?;
        Some((major.parse().ok()?, minor.parse().ok()?))
    }

    /// 主畫面要不要顯示新版本橫幅
    pub fn should_show_banner(&self, current: Version) -> bool {
        match self.availability(current) {
            Availability::Feature(v) => self.skipped_line() != Some(v.feature_line()),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V064: Version = Version::new(0, 6, 4);

    fn state_with(latest: &str) -> UpdateState {
        UpdateState { latest: Some(latest.into()), ..Default::default() }
    }

    #[test]
    fn availability_rules() {
        assert_eq!(availability(V064, Some(Version::new(0, 7, 0))), Availability::Feature(Version::new(0, 7, 0)));
        assert_eq!(availability(V064, Some(Version::new(1, 0, 0))), Availability::Feature(Version::new(1, 0, 0)));
        assert_eq!(availability(V064, Some(Version::new(0, 6, 5))), Availability::Patch(Version::new(0, 6, 5)));
        assert_eq!(availability(V064, Some(V064)), Availability::UpToDate);
        assert_eq!(availability(V064, Some(Version::new(0, 6, 3))), Availability::UpToDate);
        assert_eq!(availability(V064, Some(Version::new(0, 5, 9))), Availability::UpToDate);
        assert_eq!(availability(V064, None), Availability::UpToDate);
    }

    #[test]
    fn banner_only_for_feature_versions() {
        assert!(state_with("0.7.0").should_show_banner(V064));
        assert!(!state_with("0.6.5").should_show_banner(V064));
        assert!(!state_with("0.6.4").should_show_banner(V064));
        assert!(!state_with("garbage").should_show_banner(V064));
        assert!(!UpdateState::default().should_show_banner(V064));
    }

    #[test]
    fn skip_covers_the_whole_minor_line() {
        let mut s = state_with("0.7.0");
        s.skip(Version::new(0, 7, 0));
        assert_eq!(s.skipped.as_deref(), Some("0.7"));
        assert!(!s.should_show_banner(V064));
        s.latest = Some("0.7.3".into());
        assert!(!s.should_show_banner(V064));
        s.latest = Some("0.8.0".into());
        assert!(s.should_show_banner(V064));
        // 略過不影響「關於」頁的真實狀態
        s.latest = Some("0.7.3".into());
        assert_eq!(s.availability(V064), Availability::Feature(Version::new(0, 7, 3)));
    }

    #[test]
    fn check_interval() {
        let mut s = UpdateState::default();
        assert!(s.check_due(1000));
        s.record_check(1000, &Manifest { version: "0.7.0".into(), ..Default::default() });
        assert_eq!(s.latest.as_deref(), Some("0.7.0"));
        assert!(!s.check_due(1000 + CHECK_INTERVAL_SECS - 1));
        assert!(s.check_due(1000 + CHECK_INTERVAL_SECS));
        assert!(s.check_due(500)); // 時鐘往回調
        s.auto_check = false;
        assert!(!s.check_due(1000 + CHECK_INTERVAL_SECS * 2));
    }

    #[test]
    fn record_invalid_manifest_version_clears_latest() {
        let mut s = state_with("0.7.0");
        s.record_check(1, &Manifest { version: "0.8.0-rc.1".into(), ..Default::default() });
        assert_eq!(s.latest, None);
    }

    #[test]
    fn state_serde_defaults() {
        let s: UpdateState = serde_json::from_str("{}").unwrap();
        assert_eq!(s, UpdateState::default());
        assert!(s.auto_check);
        let s: UpdateState = serde_json::from_str(r#"{"skipped":"0.7","unknown":1}"#).unwrap();
        assert_eq!(s.skipped.as_deref(), Some("0.7"));
    }
}
