use crate::gacha::{FruitDef, Rarity};

pub const DUPLICATE_LIMIT_WITH_DLC: u32 = 3;

pub fn default_fruit_pool() -> Vec<FruitDef> {
    vec![
        FruitDef::new("Rocket", Rarity::Common, 40),
        FruitDef::new("Spin", Rarity::Common, 30),
        FruitDef::new("Flame", Rarity::Rare, 18),
        FruitDef::new("Ice", Rarity::Rare, 16),
        FruitDef::new("Light", Rarity::Epic, 10),
        FruitDef::new("Magma", Rarity::Epic, 7),
        FruitDef::new("Dragon", Rarity::Legendary, 3),
    ]
}
