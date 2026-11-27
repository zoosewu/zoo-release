//! 內建的簽章公鑰（docs/SIGNING.md）。驗證時任一把通過即可。
//!
//! 換金鑰：新公鑰先放進 `NEXT` 發布一版，等使用者更新後再換 CI 的私鑰，之後把它移到 `CURRENT`。

/// 目前的公鑰（Tauri 格式：minisign 公鑰檔再做 base64）。
/// 沿用 zoo-downloader 原有的金鑰（金鑰 ID 6D1CE0BB9F067C60）。
pub const CURRENT: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDZEMUNFMEJCOUYwNjdDNjAKUldSZ2ZBYWZ1K0FjYmFPdnpUYTlJUXZVQnpHdklEVXRES3crTEtMbnBqQkhiWW1OTkIvcjlqSngK";

/// 下一把公鑰；沒有換金鑰計畫時為空字串
pub const NEXT: &str = "";

/// 傳給驗證函式的清單（空字串會被略過）
pub const ALL: &[&str] = &[CURRENT, NEXT];

#[cfg(test)]
mod tests {
    #[test]
    fn builtin_keys_decode() {
        // 簽章不符，但不是公鑰格式錯誤
        let sig = include_str!("../tests/fixtures/zoo-downloader_0.6.0_x64-setup.exe.sig");
        assert_eq!(crate::verify_signature(b"x", sig, super::ALL), Err(crate::Error::Signature));
    }
}
