use rand::Rng;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FruitDef {
    pub name: &'static str,
    pub rarity: Rarity,
    pub weight: u32,
}

impl FruitDef {
    pub fn new(name: &'static str, rarity: Rarity, weight: u32) -> Self {
        Self {
            name,
            rarity,
            weight,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl fmt::Display for Rarity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Rarity::Common => "Common",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
        };

        write!(f, "{label}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DlcState {
    pub duplicate_limit: u32,
}

impl DlcState {
    pub fn disabled() -> Self {
        Self { duplicate_limit: 1 }
    }

    pub fn enabled(duplicate_limit: u32) -> Self {
        Self {
            duplicate_limit: duplicate_limit.max(1),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.duplicate_limit > 1
    }
}

#[derive(Debug, Default)]
pub struct PlayerInventory {
    items: BTreeMap<&'static str, u32>,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn quantity_of(&self, fruit_name: &'static str) -> u32 {
        self.items.get(fruit_name).copied().unwrap_or(0)
    }

    pub fn limit_for(&self, _fruit: &FruitDef, dlc_state: DlcState) -> u32 {
        dlc_state.duplicate_limit
    }

    pub fn add_fruit(&mut self, fruit: &FruitDef, dlc_state: DlcState) -> AddFruitResult {
        let current = self.quantity_of(fruit.name);
        let limit = self.limit_for(fruit, dlc_state);

        if current >= limit {
            return AddFruitResult::DuplicateLimitReached {
                current,
                limit,
            };
        }

        self.items.insert(fruit.name, current + 1);
        AddFruitResult::Stored { new_total: current + 1 }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&'static str, u32)> + '_ {
        self.items.iter().map(|(name, qty)| (*name, *qty))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddFruitResult {
    Stored { new_total: u32 },
    DuplicateLimitReached { current: u32, limit: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawOutcome {
    pub fruit: FruitSummary,
    pub storage: AddFruitResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FruitSummary {
    pub name: &'static str,
    pub rarity: Rarity,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DrawError {
    EmptyPool,
    InvalidWeights,
}

pub fn draw_fruit<R: Rng + ?Sized>(
    inventory: &mut PlayerInventory,
    pool: &[FruitDef],
    dlc_state: DlcState,
    rng: &mut R,
) -> Result<DrawOutcome, DrawError> {
    let fruit = choose_weighted(pool, rng)?;
    let storage = inventory.add_fruit(fruit, dlc_state);

    Ok(DrawOutcome {
        fruit: FruitSummary {
            name: fruit.name,
            rarity: fruit.rarity,
        },
        storage,
    })
}

fn choose_weighted<'a, R: Rng + ?Sized>(
    pool: &'a [FruitDef],
    rng: &mut R,
) -> Result<&'a FruitDef, DrawError> {
    if pool.is_empty() {
        return Err(DrawError::EmptyPool);
    }

    let total_weight: u32 = pool.iter().map(|fruit| fruit.weight).sum();
    if total_weight == 0 {
        return Err(DrawError::InvalidWeights);
    }

    let mut roll = rng.gen_range(0..total_weight);
    for fruit in pool {
        if roll < fruit.weight {
            return Ok(fruit);
        }
        roll -= fruit.weight;
    }

    Err(DrawError::InvalidWeights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[derive(Debug)]
    struct SequenceRng {
        values: Vec<u64>,
        index: usize,
    }

    impl SequenceRng {
        fn new(values: Vec<u64>) -> Self {
            Self { values, index: 0 }
        }
    }

    impl RngCore for SequenceRng {
        fn next_u32(&mut self) -> u32 {
            self.next_u64() as u32
        }

        fn next_u64(&mut self) -> u64 {
            let value = self.values.get(self.index).copied().unwrap_or(0);
            self.index += 1;
            value
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let mut filled = 0;
            while filled < dest.len() {
                let chunk = self.next_u64().to_le_bytes();
                let remaining = dest.len() - filled;
                let take = remaining.min(chunk.len());
                dest[filled..filled + take].copy_from_slice(&chunk[..take]);
                filled += take;
            }
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    fn single_fruit_pool() -> Vec<FruitDef> {
        vec![FruitDef::new("Rocket", Rarity::Common, 1)]
    }

    #[test]
    fn first_draw_stores_fruit() {
        let mut inventory = PlayerInventory::new();
        let mut rng = SequenceRng::new(vec![0]);

        let outcome =
            draw_fruit(&mut inventory, &single_fruit_pool(), DlcState::disabled(), &mut rng)
                .unwrap();

        assert_eq!(outcome.fruit.name, "Rocket");
        assert_eq!(outcome.storage, AddFruitResult::Stored { new_total: 1 });
        assert_eq!(inventory.quantity_of("Rocket"), 1);
    }

    #[test]
    fn duplicate_draw_fails_without_dlc() {
        let mut inventory = PlayerInventory::new();
        let mut rng = SequenceRng::new(vec![0, 0]);

        draw_fruit(&mut inventory, &single_fruit_pool(), DlcState::disabled(), &mut rng).unwrap();
        let outcome =
            draw_fruit(&mut inventory, &single_fruit_pool(), DlcState::disabled(), &mut rng)
                .unwrap();

        assert_eq!(
            outcome.storage,
            AddFruitResult::DuplicateLimitReached {
                current: 1,
                limit: 1,
            }
        );
        assert_eq!(inventory.quantity_of("Rocket"), 1);
    }

    #[test]
    fn duplicate_draw_succeeds_until_dlc_limit() {
        let mut inventory = PlayerInventory::new();
        let mut rng = SequenceRng::new(vec![0, 0, 0, 0]);
        let dlc = DlcState::enabled(3);

        let first = draw_fruit(&mut inventory, &single_fruit_pool(), dlc, &mut rng).unwrap();
        let second = draw_fruit(&mut inventory, &single_fruit_pool(), dlc, &mut rng).unwrap();
        let third = draw_fruit(&mut inventory, &single_fruit_pool(), dlc, &mut rng).unwrap();
        let fourth = draw_fruit(&mut inventory, &single_fruit_pool(), dlc, &mut rng).unwrap();

        assert_eq!(first.storage, AddFruitResult::Stored { new_total: 1 });
        assert_eq!(second.storage, AddFruitResult::Stored { new_total: 2 });
        assert_eq!(third.storage, AddFruitResult::Stored { new_total: 3 });
        assert_eq!(
            fourth.storage,
            AddFruitResult::DuplicateLimitReached {
                current: 3,
                limit: 3,
            }
        );
    }

    #[test]
    fn different_fruits_are_stored_independently() {
        let rocket = FruitDef::new("Rocket", Rarity::Common, 1);
        let spin = FruitDef::new("Spin", Rarity::Common, 1);
        let mut inventory = PlayerInventory::new();
        let first = inventory.add_fruit(&rocket, DlcState::disabled());
        let second = inventory.add_fruit(&spin, DlcState::disabled());

        assert_eq!(first, AddFruitResult::Stored { new_total: 1 });
        assert_eq!(second, AddFruitResult::Stored { new_total: 1 });
        assert_eq!(inventory.quantity_of("Rocket"), 1);
        assert_eq!(inventory.quantity_of("Spin"), 1);
    }

    #[test]
    fn empty_pool_returns_error() {
        let mut inventory = PlayerInventory::new();
        let mut rng = SequenceRng::new(vec![0]);

        let result = draw_fruit(&mut inventory, &[], DlcState::disabled(), &mut rng);

        assert_eq!(result, Err(DrawError::EmptyPool));
    }
}
