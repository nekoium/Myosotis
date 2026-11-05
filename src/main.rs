#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod document;
#[cfg(any(windows, test))]
mod input;
#[cfg(any(windows, test))]
mod settings;
mod stroke;
#[cfg(windows)]
mod windows;

fn main() {
    #[cfg(windows)]
    if let Err(error) = windows::run() {
        windows::show_error(&format!("Myosotis could not start.\n\n{error}"));
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    eprintln!("The Myosotis MVP currently supports Windows only.");
}
