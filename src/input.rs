use std::{io, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Action {
    Pen,
    Arrow,
    Rectangle,
    Number,
    Copy,
    Cancel,
    Undo,
    Redo,
    Clear,
}
impl Action {
    pub const ALL: [Self; 9] = [
        Self::Pen,
        Self::Arrow,
        Self::Rectangle,
        Self::Number,
        Self::Copy,
        Self::Cancel,
        Self::Undo,
        Self::Redo,
        Self::Clear,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Pen => "Pen",
            Self::Arrow => "Arrow",
            Self::Rectangle => "Rectangle",
            Self::Number => "Numbered mark",
            Self::Copy => "Copy & close",
            Self::Cancel => "Cancel",
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Clear => "Clear annotations",
        }
    }
    fn setting(self) -> &'static str {
        [
            "pen",
            "arrow",
            "rectangle",
            "number",
            "copy",
            "cancel",
            "undo",
            "redo",
            "clear",
        ][self as usize]
    }
}

// Values match Win32 MOD_*; keep the saved model independent of OS handles.
pub const ALT: u32 = 1;
pub const CTRL: u32 = 2;
pub const SHIFT: u32 = 4;
pub const WIN: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub key: u16,
    pub modifiers: u32,
}
impl Default for Shortcut {
    fn default() -> Self {
        Self {
            key: b'S' as u16,
            modifiers: CTRL | ALT,
        }
    }
}
impl Shortcut {
    pub fn label(self) -> String {
        let mut parts = Vec::new();
        for (flag, label) in [(CTRL, "Ctrl"), (ALT, "Alt"), (SHIFT, "Shift"), (WIN, "Win")] {
            if self.modifiers & flag != 0 {
                parts.push(label.to_owned());
            }
        }
        parts.push(key_label(self.key));
        parts.join(" + ")
    }
    pub fn validate(self) -> Result<(), String> {
        if !supported_key(self.key)
            || is_modifier(self.key)
            || self.modifiers & !15 != 0
            || self.modifiers & (CTRL | ALT | WIN) == 0
        {
            return Err("Use Ctrl, Alt, or Win with a letter, number, or function key.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings {
    pub capture: Shortcut,
    keys: [u16; 9],
}
impl Default for Bindings {
    fn default() -> Self {
        Self {
            capture: Shortcut::default(),
            keys: [0x31, 0x32, 0x33, 0x34, 0x11, 0x1b, 0x5a, 0x59, 0x52],
        }
    }
}
impl Bindings {
    pub fn key(&self, action: Action) -> u16 {
        self.keys[action as usize]
    }
    pub fn label(&self, action: Action) -> String {
        key_label(self.key(action))
    }
    pub fn action(&self, key: u16) -> Option<Action> {
        Action::ALL
            .into_iter()
            .find(|&action| self.key(action) == normalize_key(key))
    }
    pub fn set(&mut self, action: Action, key: u16) -> Result<(), String> {
        let key = normalize_key(key);
        if !supported_key(key) || matches!(key, 0x09 | 0x0d | 0x20 | 0x10 | 0x12 | 0x5b | 0x5c) {
            return Err(
                "Use a single key. Space, Enter, Tab, Shift, Alt, and Win are reserved.".into(),
            );
        }
        if let Some(other) = self.action(key).filter(|&other| other != action) {
            return Err(format!("{} is used by {}.", key_label(key), other.name()));
        }
        self.keys[action as usize] = key;
        Ok(())
    }
    pub fn load(path: &Path) -> io::Result<Self> {
        let table = crate::settings::read_table(path)?;
        let Some(value) = table.get("input") else {
            return Ok(Self::default());
        };
        let invalid = || {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid input shortcuts; using defaults",
            )
        };
        let table = value.as_table().ok_or_else(invalid)?;
        let mut result = Self::default();
        for action in Action::ALL {
            if let Some(value) = table.get(action.setting()) {
                result.keys[action as usize] = value
                    .as_integer()
                    .and_then(|key| u16::try_from(key).ok())
                    .ok_or_else(invalid)?;
            }
        }
        if let Some(value) = table.get("capture_key") {
            result.capture.key = value
                .as_integer()
                .and_then(|key| u16::try_from(key).ok())
                .ok_or_else(invalid)?;
        }
        if let Some(value) = table.get("capture_modifiers") {
            result.capture.modifiers = value
                .as_integer()
                .and_then(|key| u32::try_from(key).ok())
                .ok_or_else(invalid)?;
        }
        result.capture.validate().map_err(|_| invalid())?;
        for action in Action::ALL {
            result
                .set(action, result.key(action))
                .map_err(|_| invalid())?;
        }
        Ok(result)
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut table = crate::settings::read_table(path)?;
        let input = table
            .entry("input")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let input = input.as_table_mut().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "The input setting is not a table",
            )
        })?;
        input.insert(
            "capture_key".into(),
            toml::Value::Integer(self.capture.key as i64),
        );
        input.insert(
            "capture_modifiers".into(),
            toml::Value::Integer(self.capture.modifiers as i64),
        );
        for action in Action::ALL {
            input.insert(
                action.setting().into(),
                toml::Value::Integer(self.key(action) as i64),
            );
        }
        crate::settings::save_table(path, &table)
    }
}
pub fn normalize_key(key: u16) -> u16 {
    match key {
        0xa0 | 0xa1 => 0x10,
        0xa2 | 0xa3 => 0x11,
        0xa4 | 0xa5 => 0x12,
        _ => key,
    }
}
pub fn is_modifier(key: u16) -> bool {
    matches!(normalize_key(key), 0x10..=0x12 | 0x5b | 0x5c)
}
fn supported_key(key: u16) -> bool {
    matches!(key, 0x08 | 0x09 | 0x0d | 0x10..=0x14 | 0x1b | 0x20..=0x28 | 0x2d | 0x2e | 0x30..=0x39 | 0x41..=0x5a | 0x60..=0x6f | 0x70..=0x87 | 0x90 | 0x91 | 0xba..=0xc0 | 0xdb..=0xde)
}
pub fn key_label(key: u16) -> String {
    match key {
        0x30..=0x39 | 0x41..=0x5a => char::from_u32(key as u32).unwrap().to_string(),
        0x70..=0x87 => format!("F{}", key - 0x70 + 1),
        0x60..=0x69 => format!("Num {}", key - 0x60),
        _ => match key {
            0x08 => "Backspace",
            0x09 => "Tab",
            0x0d => "Enter",
            0x10 => "Shift",
            0x11 => "Ctrl",
            0x12 => "Alt",
            0x13 => "Pause",
            0x14 => "Caps Lock",
            0x1b => "Esc",
            0x20 => "Space",
            0x21 => "PgUp",
            0x22 => "PgDn",
            0x23 => "End",
            0x24 => "Home",
            0x25 => "←",
            0x26 => "↑",
            0x27 => "→",
            0x28 => "↓",
            0x2d => "Insert",
            0x2e => "Delete",
            0x6a => "Num *",
            0x6b => "Num +",
            0x6c => "Num ,",
            0x6d => "Num −",
            0x6e => "Num .",
            0x6f => "Num /",
            0x90 => "Num Lock",
            0x91 => "Scroll Lock",
            0xba => ";",
            0xbb => "=",
            0xbc => ",",
            0xbd => "−",
            0xbe => ".",
            0xbf => "/",
            0xc0 => "`",
            0xdb => "[",
            0xdc => "\\",
            0xdd => "]",
            0xde => "'",
            _ => "?",
        }
        .into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_conflicts_reserved_keys_and_invalid_capture() {
        let mut bindings = Bindings::default();
        assert!(bindings.set(Action::Pen, 0x32).is_err());
        for key in [0x20, 0x0d, 0x09, 0x10, 0x12, 0x5b, 0xff] {
            assert!(bindings.set(Action::Pen, key).is_err());
        }
        bindings.set(Action::Pen, 0x51).unwrap();
        assert_eq!(bindings.action(0x51), Some(Action::Pen));
        assert_eq!(bindings.action(0x31), None);
        assert_eq!(bindings.action(0xa3), Some(Action::Copy));
        assert!(Shortcut {
            key: 0x51,
            modifiers: SHIFT
        }
        .validate()
        .is_err());
        assert!(Shortcut {
            key: 0x11,
            modifiers: CTRL
        }
        .validate()
        .is_err());
        assert_eq!(Shortcut::default().label(), "Ctrl + Alt + S");
    }
    #[test]
    fn persistence_reset_and_failed_writes_preserve_other_settings() {
        let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(".tmp")
            .join(format!("input-settings-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("settings.toml");
        assert_eq!(Bindings::load(&path).unwrap(), Bindings::default());
        assert!(!path.exists());
        std::fs::write(
            &path,
            "theme = 'light'\nother = true\n[input]\nfuture = 'keep'\n",
        )
        .unwrap();
        let mut bindings = Bindings::default();
        // Swapping existing keys must load without validating against half-loaded defaults.
        bindings.keys.swap(0, 1);
        bindings.capture.key = 0x46;
        bindings.save(&path).unwrap();
        assert_eq!(Bindings::load(&path).unwrap(), bindings);
        Bindings::default().save(&path).unwrap();
        assert_eq!(Bindings::load(&path).unwrap(), Bindings::default());
        let table = crate::settings::read_table(&path).unwrap();
        assert_eq!(table["theme"].as_str(), Some("light"));
        assert_eq!(table["other"].as_bool(), Some(true));
        assert_eq!(table["input"]["future"].as_str(), Some("keep"));
        let original = std::fs::read_to_string(&path).unwrap();
        std::fs::create_dir(path.with_extension("toml.tmp")).unwrap();
        assert!(bindings.save(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        std::fs::remove_dir(path.with_extension("toml.tmp")).unwrap();
        for text in [
            "theme = [",
            "[input]\npen = 50",
            "[input]\ncopy = 32",
            "[input]\ncapture_modifiers = 0",
            "input = false",
        ] {
            std::fs::write(&path, text).unwrap();
            assert!(Bindings::load(&path).is_err(), "{text}");
        }
        assert!(bindings.save(&path).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
