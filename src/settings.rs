use std::{fs, io, path::Path};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    pub fn load(path: &Path) -> io::Result<Self> {
        let table = read_table(path)?;
        Ok(match table.get("theme").and_then(toml::Value::as_str) {
            Some("light") => Self::Light,
            _ => Self::Dark,
        })
    }

    pub fn save(self, path: &Path) -> io::Result<()> {
        let mut table = read_table(path)?;
        table.insert(
            "theme".into(),
            toml::Value::String(match self {
                Self::Dark => "dark".into(),
                Self::Light => "light".into(),
            }),
        );
        save_table(path, &table)
    }
}

pub(crate) fn save_table(path: &Path, table: &toml::Table) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // Replace only after a complete write, preserving the previous settings on failure.
    let temporary = path.with_extension("toml.tmp");
    let result =
        fs::write(&temporary, table.to_string()).and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(crate) fn read_table(path: &Path) -> io::Result<toml::Table> {
    match fs::read_to_string(path) {
        Ok(text) => text
            .parse()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(toml::Table::new()),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
pub fn path() -> io::Result<std::path::PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .filter(|path| path.is_absolute())
        .map(|path| path.join("Myosotis").join("settings.toml"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is unavailable"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trips_and_failed_saves_preserve_settings() {
        let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("theme-settings-{}", std::process::id()));
        let path = directory.join("settings.toml");
        assert!(!path.exists());
        assert_eq!(Theme::load(&path).unwrap(), Theme::Dark);
        assert!(!directory.exists());
        Theme::Light.save(&path).unwrap();
        assert_eq!(Theme::load(&path).unwrap(), Theme::Light);
        Theme::Dark.save(&path).unwrap();
        assert_eq!(Theme::load(&path).unwrap(), Theme::Dark);

        for text in ["", "theme = 'unknown'", "theme = 42"] {
            fs::write(&path, text).unwrap();
            assert_eq!(Theme::load(&path).unwrap(), Theme::Dark);
        }
        fs::write(&path, "other = true\ntheme = 'light'\n").unwrap();
        Theme::Dark.save(&path).unwrap();
        assert_eq!(read_table(&path).unwrap()["other"].as_bool(), Some(true));

        let original = fs::read_to_string(&path).unwrap();
        let temporary = path.with_extension("toml.tmp");
        fs::create_dir(&temporary).unwrap();
        assert!(Theme::Light.save(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        fs::remove_dir(&temporary).unwrap();

        fs::write(&path, "theme = [").unwrap();
        assert!(Theme::load(&path).is_err());
        assert!(Theme::Light.save(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "theme = [");
        fs::remove_dir_all(directory).unwrap();
    }
}
