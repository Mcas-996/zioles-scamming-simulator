use crate::gacha::{FruitDef, Rarity};
use std::fs;
use std::io;
use std::path::Path;

pub const DLC_CODE_PATH: &str = "DLC/p1fs.txt";
pub const DUPLICATE_LIMIT_INCREMENT_PER_DLC_CODE: u32 = 1;

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

    fn write_temp_codes(contents: &str) -> std::path::PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("one_luck_dlc_codes_{id}.txt"));
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
}
