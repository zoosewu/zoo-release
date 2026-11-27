//! zoosewu 各 app 共用的更新邏輯（規格：docs/UPDATE.md）。
//!
//! 只處理「判斷」與「驗證」：`latest.json` 解析、版本比較、要不要在主畫面提示、何時檢查、
//! 簽章與 sha256 驗證。HTTP、下載進度、安裝與畫面由各 app 自己實作。
//!
//! ```no_run
//! use zoo_update::{Manifest, UpdateState, Version, keys, manifest_url, current_platform};
//!
//! let current = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
//! let mut state = UpdateState::default();            // 從設定檔讀出
//! let now = 1_800_000_000;                           // Unix 秒
//! if state.check_due(now) {
//!     let json = "…";                                // app 自己用 HTTP GET manifest_url("zoosewu/z-overlay")
//!     let manifest = Manifest::parse(json).unwrap();
//!     state.record_check(now, &manifest);
//! }
//! if state.should_show_banner(current) { /* 顯示橫幅 */ }
//! // 使用者按「更新」後：
//! # let manifest = Manifest::parse("{}").unwrap();
//! # let data: Vec<u8> = vec![];
//! let asset = manifest.asset(current_platform()).unwrap();
//! asset.check_url("zoosewu/z-overlay").unwrap();
//! asset.verify(&data, keys::ALL).unwrap();           // 下載的內容
//! ```

mod manifest;
mod state;
mod verify;
mod version;

pub mod keys;

pub use manifest::{Manifest, PlatformAsset, current_platform, manifest_url, release_page_url};
pub use state::{Availability, UpdateState, availability};
pub use verify::{Error, sha256_hex, verify_signature, verify_signature_file};
pub use version::Version;
