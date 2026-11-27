use std::fmt;

/// `主.次.修補`。接受開頭的 `v`；預先發行版（`0.7.0-rc.1`）、`+build` 與其他格式一律視為無效，
/// 讀到時當作「沒有新版」。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self { major, minor, patch }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let s = s.strip_prefix('v').unwrap_or(s);
        let mut parts = s.split('.');
        let mut next = || -> Option<u64> {
            let p = parts.next()?;
            // 不接受空字串、+、前導 0（"01"）以外的數字格式
            if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) || (p.len() > 1 && p.starts_with('0')) {
                return None;
            }
            p.parse().ok()
        };
        let v = Version::new(next()?, next()?, next()?);
        parts.next().is_none().then_some(v)
    }

    /// 用來比較「主畫面要不要提示」與「略過」的 `主.次`
    pub fn feature_line(self) -> (u64, u64) {
        (self.major, self.minor)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(Version::parse("0.7.0"), Some(Version::new(0, 7, 0)));
        assert_eq!(Version::parse("v1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(Version::parse(" 10.20.30 "), Some(Version::new(10, 20, 30)));
        for bad in ["", "v", "1", "1.2", "1.2.3.4", "1.2.3-rc.1", "1.2.3+b", "01.2.3", "1.x.3", "-1.2.3", "1..3"] {
            assert_eq!(Version::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn order_and_display() {
        assert!(Version::new(0, 10, 0) > Version::new(0, 9, 9));
        assert!(Version::new(1, 0, 0) > Version::new(0, 99, 99));
        assert_eq!(Version::new(0, 7, 1).to_string(), "0.7.1");
    }
}
