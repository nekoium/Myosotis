use std::{io, os::windows::ffi::OsStrExt, path::Path};
use windows::{
    core::{w, PCWSTR},
    Win32::{Foundation::*, System::Registry::*},
};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

pub fn disabled_in_windows() -> bool {
    disabled_at(APPROVED_KEY)
}

fn disabled_at(path: &str) -> bool {
    let Ok(Some(key)) = Key::open(path, KEY_QUERY_VALUE) else {
        return false;
    };
    let mut data = [0u8; 12];
    let mut size = data.len() as u32;
    let mut kind = REG_VALUE_TYPE::default();
    let result = unsafe {
        RegQueryValueExW(
            key.0,
            w!("Myosotis"),
            None,
            Some(&mut kind),
            Some(data.as_mut_ptr()),
            Some(&mut size),
        )
    };
    // shortcut: StartupApproved is undocumented; recognize only the known disabled record, revisit if Windows changes it.
    result == ERROR_SUCCESS
        && kind == REG_BINARY
        && size == data.len() as u32
        && u32::from_le_bytes(data[..4].try_into().unwrap()) == 3
}

pub fn enabled() -> io::Result<bool> {
    enabled_at(RUN_KEY)
}

pub fn set_enabled(enabled: bool) -> io::Result<()> {
    if enabled {
        let command = startup_command(&std::env::current_exe()?);
        Key::create(RUN_KEY)?.set(&command)
    } else {
        match Key::open(RUN_KEY, KEY_SET_VALUE)? {
            Some(key) => key.remove(),
            None => Ok(()),
        }
    }
}

fn startup_command(exe: &Path) -> Vec<u16> {
    // Quote the executable even without spaces; login must never parse a partial path.
    std::iter::once(b'"' as u16)
        .chain(exe.as_os_str().encode_wide())
        .chain([b'"' as u16, 0])
        .collect()
}

fn enabled_at(path: &str) -> io::Result<bool> {
    match Key::open(path, KEY_QUERY_VALUE)? {
        Some(key) => {
            match unsafe { RegQueryValueExW(key.0, w!("Myosotis"), None, None, None, None) } {
                ERROR_SUCCESS => Ok(true),
                ERROR_FILE_NOT_FOUND => Ok(false),
                error => Err(io::Error::from_raw_os_error(error.0 as i32)),
            }
        }
        None => Ok(false),
    }
}

struct Key(HKEY);
impl Key {
    fn open(path: &str, access: REG_SAM_FLAGS) -> io::Result<Option<Self>> {
        let path: Vec<u16> = path.encode_utf16().chain([0]).collect();
        let mut key = HKEY::default();
        match unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                0,
                access,
                &mut key,
            )
        } {
            ERROR_SUCCESS => Ok(Some(Self(key))),
            ERROR_FILE_NOT_FOUND => Ok(None),
            error => Err(io::Error::from_raw_os_error(error.0 as i32)),
        }
    }

    fn create(path: &str) -> io::Result<Self> {
        let path: Vec<u16> = path.encode_utf16().chain([0]).collect();
        let mut key = HKEY::default();
        unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                0,
                PCWSTR::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                None,
                &mut key,
                None,
            )
            .ok()?;
        }
        Ok(Self(key))
    }

    fn set(&self, command: &[u16]) -> io::Result<()> {
        let bytes: Vec<u8> = command.iter().flat_map(|unit| unit.to_le_bytes()).collect();
        unsafe { RegSetValueExW(self.0, w!("Myosotis"), 0, REG_SZ, Some(&bytes)).ok()? };
        Ok(())
    }

    fn remove(&self) -> io::Result<()> {
        match unsafe { RegDeleteValueW(self.0, w!("Myosotis")) } {
            ERROR_SUCCESS | ERROR_FILE_NOT_FOUND => Ok(()),
            error => Err(io::Error::from_raw_os_error(error.0 as i32)),
        }
    }
}
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_quotes_unicode_path_without_capture_argument() {
        let command = startup_command(Path::new(r"C:\My apps\花\myosotis.exe"));
        assert_eq!(
            String::from_utf16(&command[..command.len() - 1]).unwrap(),
            "\"C:\\My apps\\花\\myosotis.exe\""
        );
        assert_eq!(command.last(), Some(&0));
    }

    #[test]
    fn registration_round_trip_and_denied_writes() {
        // Never touch the real Run key, even when tests fail.
        let path = format!(r"Software\Myosotis-Startup-Test-{}", std::process::id());
        struct Cleanup(String);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let path: Vec<u16> = self.0.encode_utf16().chain([0]).collect();
                unsafe {
                    let _ = RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(path.as_ptr()));
                }
            }
        }
        assert!(Key::open(&path, KEY_QUERY_VALUE).unwrap().is_none());
        let _cleanup = Cleanup(path.clone());
        assert!(!enabled_at(&path).unwrap());
        assert!(!disabled_at(&path));
        assert!(Key::open(&path, KEY_QUERY_VALUE).unwrap().is_none());
        let command = startup_command(Path::new(r"C:\My apps\花\myosotis.exe"));
        let key = Key::create(&path).unwrap();
        assert!(!enabled_at(&path).unwrap());
        assert!(!disabled_at(&path));
        key.set(&command).unwrap();
        assert!(!disabled_at(&path));
        unsafe {
            RegSetValueExW(key.0, w!("Other app"), 0, REG_SZ, Some(&[0, 0]))
                .ok()
                .unwrap();
        }
        assert!(enabled_at(&path).unwrap());
        let reader = Key::open(&path, KEY_QUERY_VALUE).unwrap().unwrap();
        let mut actual = vec![0u16; command.len()];
        let mut size = (actual.len() * 2) as u32;
        unsafe {
            RegQueryValueExW(
                reader.0,
                w!("Myosotis"),
                None,
                None,
                Some(actual.as_mut_ptr().cast()),
                Some(&mut size),
            )
            .ok()
            .unwrap();
        }
        assert_eq!(actual, command);
        assert!(reader.set(&command).is_err());
        assert!(reader.remove().is_err());
        assert!(enabled_at(&path).unwrap());
        drop(reader);
        drop(key);
        assert!(enabled_at(&path).unwrap());
        let key = Key::open(&path, KEY_SET_VALUE).unwrap().unwrap();
        key.remove().unwrap();
        assert!(!enabled_at(&path).unwrap());
        key.remove().unwrap();
        let reader = Key::open(&path, KEY_QUERY_VALUE).unwrap().unwrap();
        unsafe {
            RegQueryValueExW(reader.0, w!("Other app"), None, None, None, None)
                .ok()
                .unwrap();
        }
        assert!(!disabled_at(&path));
        let mut disabled = [0u8; 12];
        disabled[0] = 3;
        // Windows stores a timestamp after the four-byte state.
        disabled[4..].fill(42);
        let enabled = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        for (kind, bytes, expected) in [
            (REG_BINARY, disabled.as_slice(), true),
            (REG_BINARY, enabled.as_slice(), false),
            (REG_BINARY, &[0u8; 12][..], false),
            (REG_BINARY, &[255u8; 12][..], false),
            (REG_BINARY, &disabled[..1], false),
            (REG_BINARY, &[3u8; 13][..], false),
            (REG_SZ, disabled.as_slice(), false),
        ] {
            unsafe {
                RegSetValueExW(key.0, w!("Myosotis"), 0, kind, Some(bytes))
                    .ok()
                    .unwrap();
            }
            assert_eq!(disabled_at(&path), expected);
        }
        key.remove().unwrap();
        assert!(!disabled_at(&path));
    }
}
