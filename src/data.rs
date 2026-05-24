use crate::gacha::{FruitDef, Rarity};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DLC_CODE_PATH: &str = "DLC/p1fs.txt";
pub const DUPLICATE_LIMIT_INCREMENT_PER_DLC_CODE: u32 = 1;
pub const SAVE_FILE_NAME: &str = "save.txt";
pub const SAVE_VERSION: &str = "1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveData {
    pub duplicate_limit: u32,
    pub inventory: Vec<(String, u32)>,
}

pub fn dlc_code_exists(code: &str, code_path: impl AsRef<Path>) -> io::Result<bool> {
    let normalized_code = code.trim();
    if normalized_code.is_empty() {
        return Ok(false);
    }

    let contents = fs::read_to_string(code_path)?;
    Ok(contents
        .lines()
        .map(str::trim)
        .any(|stored_code| !stored_code.is_empty() && stored_code == normalized_code))
}

pub fn default_save_path() -> PathBuf {
    if let Some(appdata) = non_empty_env_path("APPDATA") {
        return appdata.join("oneLuck").join(SAVE_FILE_NAME);
    }

    if let Some(home) = non_empty_env_path("HOME") {
        return home.join(".gatcha").join("oneLuck-save.txt");
    }

    PathBuf::from(".gatcha").join("oneLuck-save.txt")
}

pub fn load_save(path: impl AsRef<Path>) -> io::Result<Option<SaveData>> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path)?;
    parse_save(&contents)
        .map(Some)
        .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))
}

pub fn save_game(path: impl AsRef<Path>, save_data: &SaveData) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, serialize_save(save_data))
}

fn non_empty_env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn serialize_save(save_data: &SaveData) -> String {
    let mut output = String::new();
    output.push_str("version=");
    output.push_str(SAVE_VERSION);
    output.push('\n');
    output.push_str("duplicate_limit=");
    output.push_str(&save_data.duplicate_limit.to_string());
    output.push('\n');

    for (name, qty) in &save_data.inventory {
        if *qty == 0 {
            continue;
        }
        output.push_str("fruit=");
        output.push_str(name);
        output.push('\t');
        output.push_str(&qty.to_string());
        output.push('\n');
    }

    output
}

fn parse_save(contents: &str) -> Result<SaveData, String> {
    let mut version_seen = false;
    let mut duplicate_limit = None;
    let mut inventory = Vec::new();

    for (index, line) in contents.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(version) = line.strip_prefix("version=") {
            version_seen = true;
            if version != SAVE_VERSION {
                return Err(format!("unsupported save version `{version}`"));
            }
            continue;
        }

        if let Some(limit) = line.strip_prefix("duplicate_limit=") {
            let parsed_limit = limit.parse::<u32>().map_err(|_| {
                format!(
                    "invalid duplicate limit on save line {}",
                    index.saturating_add(1)
                )
            })?;
            duplicate_limit = Some(parsed_limit.max(1));
            continue;
        }

        if let Some(entry) = line.strip_prefix("fruit=") {
            let (name, qty) = entry.split_once('\t').ok_or_else(|| {
                format!(
                    "invalid fruit entry on save line {}",
                    index.saturating_add(1)
                )
            })?;
            let qty = qty.parse::<u32>().map_err(|_| {
                format!(
                    "invalid fruit quantity on save line {}",
                    index.saturating_add(1)
                )
            })?;
            if !name.is_empty() && qty > 0 {
                inventory.push((name.to_string(), qty));
            }
            continue;
        }

        return Err(format!(
            "unrecognized save line {}",
            index.saturating_add(1)
        ));
    }

    if !version_seen {
        return Err("save file is missing version".to_string());
    }

    Ok(SaveData {
        duplicate_limit: duplicate_limit.unwrap_or(1),
        inventory,
    })
}

pub fn default_fruit_pool() -> Vec<FruitDef> {
    vec![
        FruitDef::new("Rocket", Rarity::Common, 30.0),
        FruitDef::new("Spin", Rarity::Common, 30.0),
        FruitDef::new("diamond", Rarity::Rare, 29.0),
        FruitDef::new("flame", Rarity::Rare, 29.0),
        FruitDef::new("Light", Rarity::Epic, 10.0),
        FruitDef::new("Magma", Rarity::Epic, 10.0 - f64::from_bits(1)),
        FruitDef::new("Portal", Rarity::Legendary, 0.0001),
        FruitDef::new("Phoenix", Rarity::Legendary, 0.01),
        FruitDef::new("Piranha", Rarity::Mythical, 0.0),
        FruitDef::new("Dragon", Rarity::Mythical, f64::from_bits(1)),
    ]
}
// the third value of the FruitDef is f64, i am way too kind.

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{id}.txt"))
    }

    fn write_temp_codes(contents: &str) -> std::path::PathBuf {
        let path = unique_temp_path("one_luck_dlc_codes");
        fs::write(&path, contents).expect("test should write dlc codes");
        path
    }

    #[test]
    fn dlc_code_matches_any_non_empty_line() {
        let path = write_temp_codes("FIRST\n\nSECOND\n");

        assert!(dlc_code_exists("SECOND", &path).unwrap());
        assert!(dlc_code_exists(" FIRST ", &path).unwrap());
        assert!(!dlc_code_exists("MISSING", &path).unwrap());
        assert!(!dlc_code_exists("", &path).unwrap());

        fs::remove_file(path).expect("test should remove temp dlc codes");
    }

    #[test]
    fn save_round_trip_preserves_inventory_and_duplicate_limit() {
        let path = write_temp_codes("");
        let save_data = SaveData {
            duplicate_limit: 3,
            inventory: vec![("Rocket".to_string(), 2), ("Magma".to_string(), 1)],
        };

        save_game(&path, &save_data).unwrap();
        let loaded = load_save(&path).unwrap().unwrap();

        assert_eq!(loaded, save_data);

        fs::remove_file(path).expect("test should remove temp save");
    }

    #[test]
    fn missing_save_returns_none() {
        let path = unique_temp_path("one_luck_missing_save_for_test");

        assert_eq!(load_save(path).unwrap(), None);
    }
}
