mod data;
mod gacha;

use crate::data::{
    DLC_CODE_PATH, DUPLICATE_LIMIT_INCREMENT_PER_DLC_CODE, SaveData, default_fruit_pool,
    default_save_path,
};
use crate::gacha::{AddFruitResult, DlcState, DrawError, PlayerInventory, draw_fruit};
use rand::thread_rng;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let save_path = default_save_path();
    let (mut inventory, mut dlc_state) = load_game_state(&save_path);
    let mut rng = thread_rng();

    println!("Save file: {}", save_path.display());
    print_help();

    loop {
        print!("\n> ");
        if let Err(error) = io::stdout().flush() {
            eprintln!("failed to flush stdout: {error}");
            break;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("bye");
                break;
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("failed to read input: {error}");
                continue;
            }
        }

        let trimmed_input = input.trim();
        let mut parts = trimmed_input.splitn(2, char::is_whitespace);
        let command = parts.next().unwrap_or("").to_ascii_lowercase();
        let argument = parts.next().unwrap_or("").trim();

        match command.as_str() {
            "draw" => {
                let pool = default_fruit_pool();
                if run_draw(&mut inventory, &pool, dlc_state, &mut rng) {
                    save_current_game(&save_path, &inventory, dlc_state);
                }
            }
            "inventory" => print_inventory(&inventory, dlc_state),
            "pool" => print_pool(&default_fruit_pool()),
            "buy-dlc" => {
                let (new_dlc_state, changed) = run_buy_dlc(argument, dlc_state);
                dlc_state = new_dlc_state;
                if changed {
                    save_current_game(&save_path, &inventory, dlc_state);
                }
            }
            "help" => print_help(),
            "exit" | "quit" => {
                println!("bye");
                break;
            }
            "" => {}
            _ => println!("unknown command. type `help` to see available commands."),
        }
    }
}

fn load_game_state(save_path: &Path) -> (PlayerInventory, DlcState) {
    match data::load_save(save_path) {
        Ok(Some(save_data)) => {
            let inventory = PlayerInventory::from_entries(save_data.inventory);
            let dlc_state = DlcState {
                duplicate_limit: save_data.duplicate_limit.max(1),
            };
            println!("Loaded saved inventory.");
            (inventory, dlc_state)
        }
        Ok(None) => (PlayerInventory::new(), DlcState::disabled()),
        Err(error) => {
            eprintln!(
                "failed to load save from {}: {error}. starting with an empty inventory.",
                save_path.display()
            );
            (PlayerInventory::new(), DlcState::disabled())
        }
    }
}

fn save_current_game(save_path: &Path, inventory: &PlayerInventory, dlc_state: DlcState) {
    let save_data = SaveData {
        duplicate_limit: dlc_state.duplicate_limit,
        inventory: inventory
            .entries()
            .map(|(name, qty)| (name.to_string(), qty))
            .collect(),
    };

    if let Err(error) = data::save_game(save_path, &save_data) {
        eprintln!(
            "failed to save inventory to {}: {error}",
            save_path.display()
        );
    }
}

fn run_buy_dlc(code: &str, dlc_state: DlcState) -> (DlcState, bool) {
    if code.is_empty() {
        println!("Usage: buy-dlc <code>");
        return (dlc_state, false);
    }

    match data::dlc_code_exists(code, DLC_CODE_PATH) {
        Ok(true) => {
            let new_state =
                dlc_state.increase_duplicate_limit(DUPLICATE_LIMIT_INCREMENT_PER_DLC_CODE);
            println!(
                "DLC code accepted. Duplicate limit per fruit is now {}.",
                new_state.duplicate_limit
            );
            (new_state, true)
        }
        Ok(false) => {
            println!("Invalid DLC code.");
            (dlc_state, false)
        }
        Err(error) => {
            eprintln!("failed to read DLC codes from {DLC_CODE_PATH}: {error}");
            (dlc_state, false)
        }
    }
}

fn run_draw(
    inventory: &mut PlayerInventory,
    pool: &[gacha::FruitDef],
    dlc_state: DlcState,
    rng: &mut impl rand::Rng,
) -> bool {
    match draw_fruit(inventory, pool, dlc_state, rng) {
        Ok(outcome) => {
            println!(
                "You rolled {} [{}].",
                outcome.fruit.name, outcome.fruit.rarity
            );

            match outcome.storage {
                AddFruitResult::Stored { new_total } => {
                    println!("Stored successfully. You now hold {new_total} copy/copies.");
                    true
                }
                AddFruitResult::DuplicateLimitReached { current, limit } => {
                    println!(
                        "Storage failed. You already hold {current} copy/copies and the limit is {limit}."
                    );
                    false
                }
            }
        }
        Err(DrawError::EmptyPool) => {
            println!("The fruit pool is empty.");
            false
        }
        Err(DrawError::InvalidWeights) => {
            println!("The fruit pool weights are invalid.");
            false
        }
    }
}

fn print_inventory(inventory: &PlayerInventory, dlc_state: DlcState) {
    println!("Inventory");
    println!(
        "Duplicate DLC: {}",
        if dlc_state.is_enabled() {
            format!("enabled (limit {})", dlc_state.duplicate_limit)
        } else {
            "disabled (limit 1)".to_string()
        }
    );

    let mut has_items = false;
    for (name, qty) in inventory.entries() {
        has_items = true;
        println!("- {name}: {qty}");
    }

    if !has_items {
        println!("- empty");
    }
}

fn print_pool(pool: &[gacha::FruitDef]) {
    println!("Fruit Pool");
    for fruit in pool {
        println!(
            "- {} [{}] weight={}",
            fruit.name, fruit.rarity, fruit.weight
        );
    }
}

fn print_help() {
    println!("Commands:");
    println!("- draw      roll one fruit");
    println!("- inventory show owned fruits");
    println!("- pool      show the current fruit pool");
    println!("- buy-dlc <code> increase duplicate storage by 1 with a DLC code");
    println!("- help      show commands");
    println!("- exit      quit the demo");
}
