//! Background PNG file output, independent of clipboard ownership.
use super::clipboard::Image;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::windows::ffi::OsStringExt,
    path::{Path, PathBuf},
    sync::Arc,
    thread::JoinHandle,
};
use windows::Win32::{
    Foundation::SYSTEMTIME,
    System::{Com::CoTaskMemFree, SystemInformation::GetLocalTime},
    UI::Shell::{FOLDERID_Pictures, SHGetKnownFolderPath},
};

#[derive(Default)]
pub(super) struct Saves {
    jobs: Vec<JoinHandle<Result<PathBuf, String>>>,
}
impl Saves {
    pub(super) fn start(&mut self, image: Arc<Image>) -> io::Result<()> {
        // Timestamp completion, even if encoding or storage is slow.
        let now = unsafe { GetLocalTime() };
        let job = std::thread::Builder::new()
            .name("local-save".into())
            .spawn(move || {
                let directory = pictures()?.join("Screenshots");
                save(&directory, now, || {
                    image.png().map_err(|e| io::Error::other(e.to_string()))
                })
                .map_err(|e| format!("{}: {e}", directory.display()))
            })?;
        self.jobs.push(job);
        Ok(())
    }

    pub(super) fn collect(&mut self, wait: bool) -> Vec<String> {
        let mut errors = Vec::new();
        let mut index = 0;
        while index < self.jobs.len() {
            if wait || self.jobs[index].is_finished() {
                match self.jobs.swap_remove(index).join() {
                    Ok(Ok(path)) => eprintln!("Saved image: {}", path.display()),
                    Ok(Err(error)) => errors.push(error),
                    Err(_) => errors.push("Local save worker failed".into()),
                }
            } else {
                index += 1;
            }
        }
        errors
    }

    pub(super) fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

fn pictures() -> Result<PathBuf, String> {
    unsafe {
        let pointer = SHGetKnownFolderPath(&FOLDERID_Pictures, Default::default(), None)
            .map_err(|e| format!("Could not locate Pictures folder: {e}"))?;
        let path = PathBuf::from(std::ffi::OsString::from_wide(pointer.as_wide()));
        CoTaskMemFree(Some(pointer.0.cast()));
        Ok(path)
    }
}

fn save<'a>(
    directory: &Path,
    now: SYSTEMTIME,
    png: impl FnOnce() -> io::Result<&'a [u8]>,
) -> io::Result<PathBuf> {
    // Encode before creating any output; failed encoding leaves no image or directory.
    let bytes = png()?;
    fs::create_dir_all(directory)?;
    let stem = format!(
        "Screenshot {:04}-{:02}-{:02} {:02}{:02}{:02}",
        now.wYear, now.wMonth, now.wDay, now.wHour, now.wMinute, now.wSecond
    );
    for index in 1u64.. {
        let suffix = if index == 1 {
            String::new()
        } else {
            format!(" ({index})")
        };
        let path = directory.join(format!("{stem}{suffix}.png"));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => {
                let result = write(file, bytes);
                if result.is_err() {
                    let _ = fs::remove_file(&path);
                }
                result?;
                return Ok(path);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("No available screenshot filename"))
}

fn write(mut file: File, bytes: &[u8]) -> io::Result<()> {
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(test)]
#[path = "local_save_tests.rs"]
mod tests;
