use std::fmt;
use std::io::Read;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use minisign_verify::{PublicKey, Signature};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// `latest.json` 無法解析
    Manifest(String),
    /// 下載網址不是本 repo 的 Release
    Url(String),
    /// 公鑰格式錯誤（內建公鑰有誤，屬於程式問題）
    PublicKey,
    /// 簽章格式錯誤
    SignatureFormat,
    /// 簽章不符（檔案被改過，或不是用目前的金鑰簽的）
    Signature,
    /// sha256 不符
    Sha256,
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Manifest(e) => write!(f, "invalid latest.json: {e}"),
            Error::Url(u) => write!(f, "unexpected download URL: {u}"),
            Error::PublicKey => f.write_str("invalid built-in public key"),
            Error::SignatureFormat => f.write_str("invalid signature format"),
            Error::Signature => f.write_str("the signature didn't match"),
            Error::Sha256 => f.write_str("the SHA-256 checksum didn't match"),
            Error::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for Error {}

pub fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Tauri 的公鑰與簽章都是「minisign 文字檔」再做一次 base64
fn decode_text(b64: &str) -> Option<String> {
    let bin = STANDARD.decode(b64.trim()).ok()?;
    String::from_utf8(bin).ok()
}

fn decode_keys(public_keys: &[&str]) -> Result<Vec<PublicKey>, Error> {
    let keys: Vec<PublicKey> = public_keys
        .iter()
        .filter(|k| !k.trim().is_empty())
        .map(|k| decode_text(k).and_then(|t| PublicKey::decode(&t).ok()).ok_or(Error::PublicKey))
        .collect::<Result<_, _>>()?;
    if keys.is_empty() { Err(Error::PublicKey) } else { Ok(keys) }
}

fn decode_signature(signature: &str) -> Result<Signature, Error> {
    decode_text(signature).and_then(|t| Signature::decode(&t).ok()).ok_or(Error::SignatureFormat)
}

/// 用任一把公鑰驗證通過即可（目前＋下一把，見 docs/SIGNING.md「換金鑰」）
pub fn verify_signature(data: &[u8], signature: &str, public_keys: &[&str]) -> Result<(), Error> {
    let sig = decode_signature(signature)?;
    let keys = decode_keys(public_keys)?;
    if keys.iter().any(|k| k.verify(data, &sig, false).is_ok()) { Ok(()) } else { Err(Error::Signature) }
}

/// 逐段讀檔驗證簽章，同時算出 sha256（回傳十六進位字串）
pub fn verify_signature_file(path: &Path, signature: &str, public_keys: &[&str]) -> Result<String, Error> {
    let sig = decode_signature(signature)?;
    let keys = decode_keys(public_keys)?;
    let mut verifiers = keys.iter().filter_map(|k| k.verify_stream(&sig).ok()).collect::<Vec<_>>();
    if verifiers.is_empty() {
        return Err(Error::Signature); // 金鑰 ID 都不符，或是舊格式的簽章
    }
    let mut sha = Sha256::new();
    let mut file = std::fs::File::open(path).map_err(|e| Error::Io(e.to_string()))?;
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = file.read(&mut buf).map_err(|e| Error::Io(e.to_string()))?;
        if n == 0 {
            break;
        }
        sha.update(&buf[..n]);
        verifiers.iter_mut().for_each(|v| v.update(&buf[..n]));
    }
    if verifiers.iter_mut().any(|v| v.finalize().is_ok()) {
        Ok(hex(&sha.finalize()))
    } else {
        Err(Error::Signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// zoo-downloader v0.6.0 的公鑰與 Windows 安裝檔的簽章（真實的 Tauri 簽章）
    const PUBKEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDZEMUNFMEJCOUYwNjdDNjAKUldSZ2ZBYWZ1K0FjYmFPdnpUYTlJUXZVQnpHdklEVXRES3crTEtMbnBqQkhiWW1OTkIvcjlqSngK";
    const SIG: &str = include_str!("../tests/fixtures/zoo-downloader_0.6.0_x64-setup.exe.sig");
    /// 格式正確但無關的另一把公鑰（PUBKEY 的金鑰 ID 與金鑰內容逐位元組 XOR 0x5a）
    const OTHER_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IE9USEVSClJXUTZKbHpGNGJwR04vbjFsMnpuZTFHT1hXdjFlbTkzVnZaa2R2aTkvR29kTjlQWGJrV3hyR2dyCg==";

    #[test]
    fn sha256() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn real_tauri_formats_decode() {
        decode_keys(&[PUBKEY, OTHER_KEY]).unwrap();
        decode_signature(SIG).unwrap();
    }

    #[test]
    fn wrong_data_is_rejected() {
        assert_eq!(verify_signature(b"not the installer", SIG, &[PUBKEY]), Err(Error::Signature));
    }

    #[test]
    fn wrong_key_is_rejected() {
        assert_eq!(verify_signature(b"x", SIG, &[OTHER_KEY]), Err(Error::Signature));
    }

    #[test]
    fn bad_inputs() {
        assert_eq!(verify_signature(b"x", "not base64!", &[PUBKEY]), Err(Error::SignatureFormat));
        assert_eq!(verify_signature(b"x", SIG, &["bm90IGEga2V5"]), Err(Error::PublicKey));
        assert_eq!(verify_signature(b"x", SIG, &[]), Err(Error::PublicKey));
        assert_eq!(verify_signature(b"x", SIG, &[""]), Err(Error::PublicKey));
    }

    /// 需要真實的安裝檔（約 19 MB，不放進 repo）：
    /// ZOO_UPDATE_FIXTURE=<含 setup.exe 的資料夾> cargo test -- --ignored
    #[test]
    #[ignore]
    fn real_tauri_signature_verifies() {
        let dir = std::env::var("ZOO_UPDATE_FIXTURE").expect("ZOO_UPDATE_FIXTURE");
        let path = Path::new(&dir).join("setup.exe");
        let data = std::fs::read(&path).unwrap();
        verify_signature(&data, SIG, &[PUBKEY]).unwrap();
        // 目前＋下一把：任一把通過即可
        verify_signature(&data, SIG, &[OTHER_KEY, PUBKEY]).unwrap();
        let sha = verify_signature_file(&path, SIG, &[PUBKEY]).unwrap();
        assert_eq!(sha, sha256_hex(&data));
        let mut tampered = data.clone();
        tampered[1000] ^= 1;
        assert_eq!(verify_signature(&tampered, SIG, &[PUBKEY]), Err(Error::Signature));
    }
}
