//! Remembers the final proxy's username and password between launches.
//!
//! On Windows the pair is encrypted with DPAPI (`CryptProtectData`) for the
//! signed-in Windows user and kept in its own store file, separate from
//! `profile.json`. Another Windows account, or a copy of the file on another
//! PC, cannot decrypt it. On other platforms nothing is written and the
//! credentials stay in memory only, as before.

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

const STORE_FILE: &str = "credentials.json";
const STORE_KEY: &str = "final_proxy";

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct SavedCredentials {
    pub username: String,
    pub password: String,
}

/// True where credentials can be encrypted at rest.
pub fn supported() -> bool {
    cfg!(windows)
}

pub fn load(app: &AppHandle) -> Option<SavedCredentials> {
    use tauri_plugin_store::StoreExt;
    let stored = app.store(STORE_FILE).ok()?.get(STORE_KEY)?;
    open(stored.as_str()?)
}

pub fn save(app: &AppHandle, username: &str, password: &str) {
    use tauri_plugin_store::StoreExt;
    let Some(sealed) = seal(&SavedCredentials {
        username: username.to_owned(),
        password: password.to_owned(),
    }) else {
        return;
    };
    if let Ok(store) = app.store(STORE_FILE) {
        store.set(STORE_KEY, serde_json::Value::String(sealed));
        let _ = store.save();
    }
}

pub fn forget(app: &AppHandle) {
    use tauri_plugin_store::StoreExt;
    if let Ok(store) = app.store(STORE_FILE) {
        if store.delete(STORE_KEY) {
            let _ = store.save();
        }
    }
}

fn seal(credentials: &SavedCredentials) -> Option<String> {
    let plain = serde_json::to_vec(credentials).ok()?;
    let sealed = dpapi::protect(&plain)?;
    Some(to_hex(&sealed))
}

fn open(sealed: &str) -> Option<SavedCredentials> {
    let plain = dpapi::unprotect(&from_hex(sealed)?)?;
    serde_json::from_slice(&plain).ok()
}

fn to_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    out
}

fn from_hex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect()
}

#[cfg(windows)]
mod dpapi {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    /// Extra input to DPAPI so another program calling CryptUnprotectData on
    /// this blob also needs to know it is Aether-GUI's.
    const ENTROPY: &[u8] = b"Aether-GUI final proxy credentials v1";

    fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        }
    }

    fn empty() -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        }
    }

    /// Copies DPAPI's output and frees the buffer it allocated.
    unsafe fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        let bytes = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        LocalFree(out.pbData as _);
        bytes
    }

    pub fn protect(data: &[u8]) -> Option<Vec<u8>> {
        let input = blob(data);
        let entropy = blob(ENTROPY);
        let mut out = empty();
        // SAFETY: every pointer refers to a live local for the whole call;
        // DPAPI allocates `out` with LocalAlloc, which `take` frees.
        let ok = unsafe {
            CryptProtectData(
                &input,
                std::ptr::null(),
                &entropy,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        };
        if ok == 0 || out.pbData.is_null() {
            return None;
        }
        Some(unsafe { take(out) })
    }

    pub fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
        let input = blob(data);
        let entropy = blob(ENTROPY);
        let mut out = empty();
        // SAFETY: as in `protect`.
        let ok = unsafe {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                &entropy,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            )
        };
        if ok == 0 || out.pbData.is_null() {
            return None;
        }
        Some(unsafe { take(out) })
    }
}

#[cfg(not(windows))]
mod dpapi {
    pub fn protect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }

    pub fn unprotect(_: &[u8]) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_and_rejects_bad_input() {
        let bytes = [0u8, 1, 15, 16, 127, 128, 254, 255];
        assert_eq!(to_hex(&bytes), "00010f107f80feff");
        assert_eq!(from_hex(&to_hex(&bytes)).unwrap(), bytes);
        assert!(from_hex("abc").is_none());
        assert!(from_hex("zz").is_none());
        assert!(from_hex("é1").is_none());
    }

    #[cfg(windows)]
    #[test]
    fn dpapi_round_trips_without_storing_plain_text() {
        let credentials = SavedCredentials {
            username: "user-secret".into(),
            password: "pässword-secret".into(),
        };
        let sealed = seal(&credentials).expect("DPAPI encrypts");
        assert!(!sealed.contains(&to_hex(b"user-secret")));
        assert!(!sealed.contains(&to_hex("pässword-secret".as_bytes())));
        assert_eq!(open(&sealed), Some(credentials));
        // A damaged blob is ignored rather than crashing the app.
        let mut damaged = sealed.clone();
        damaged.replace_range(sealed.len() - 4.., "0000");
        assert_eq!(open(&damaged), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn other_platforms_do_not_store_credentials() {
        assert!(!supported());
        let credentials = SavedCredentials {
            username: "u".into(),
            password: "p".into(),
        };
        assert!(seal(&credentials).is_none());
    }
}
