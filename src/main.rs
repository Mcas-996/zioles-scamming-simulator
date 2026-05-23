mod data;
mod gacha;

use crate::data::{DUPLICATE_LIMIT_WITH_DLC, default_fruit_pool};
use crate::gacha::{AddFruitResult, DlcState, DrawError, PlayerInventory, draw_fruit};
use rand::thread_rng;
use std::io::{self, Write};

fn main() {
    let pool = default_fruit_pool();
    let mut inventory = PlayerInventory::new();
    let mut dlc_state = DlcState::disabled();
    let mut rng = thread_rng();

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

        let command = input.trim().to_ascii_lowercase();
        match command.as_str() {
            "draw" => run_draw(&mut inventory, &pool, dlc_state, &mut rng),
            "inventory" => print_inventory(&inventory, dlc_state),
            "pool" => print_pool(&pool),
            "buy-dlc" => {
                dlc_state = DlcState::enabled(DUPLICATE_LIMIT_WITH_DLC);
                println!(
                    "DLC enabled. Duplicate limit per fruit is now {}.",
                    dlc_state.duplicate_limit
                );
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

fn run_draw(
    inventory: &mut PlayerInventory,
    pool: &[gacha::FruitDef],
    dlc_state: DlcState,
    rng: &mut impl rand::Rng,
) {
    match draw_fruit(inventory, pool, dlc_state, rng) {
        Ok(outcome) => {
            println!(
                "You rolled {} [{}].",
                outcome.fruit.name, outcome.fruit.rarity
            );

            match outcome.storage {
                AddFruitResult::Stored { new_total } => {
                    println!("Stored successfully. You now hold {new_total} copy/copies.");
                }
                AddFruitResult::DuplicateLimitReached { current, limit } => {
                    println!(
                        "Storage failed. You already hold {current} copy/copies and the limit is {limit}."
                    );
                }
            }
        }
        Err(DrawError::EmptyPool) => println!("The fruit pool is empty."),
        Err(DrawError::InvalidWeights) => println!("The fruit pool weights are invalid."),
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
    println!("- buy-dlc   unlock duplicate storage");
    println!("- help      show commands");
    println!("- exit      quit the demo");
}
