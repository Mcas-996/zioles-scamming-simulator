use crate::gacha::{FruitDef, Rarity};

pub const DUPLICATE_LIMIT_WITH_DLC: u32 = 3;

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
