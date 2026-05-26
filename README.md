# oneLuck

`oneLuck` is a small Rust command-line gacha demo. It lets you roll fruits from a weighted fruit pool, store the results in an inventory, and optionally enable a DLC-style duplicate storage limit.

## Features

- Weighted fruit rolls across common, rare, epic, legendary, and mythical rarities
- Inventory tracking for owned fruits
- Duplicate prevention by default
- DLC command that accepts a code from `DLC/p1fs.txt` and raises the duplicate storage limit by 1
- Unit tests for draw behavior, duplicate limits, and empty pools

## Requirements

- Rust with Cargo installed

## Getting Started

Build and run the interactive demo:

```bash
cargo run
```

Run the test suite:

```bash
cargo test
```

## Commands

After starting the program with `cargo run`, use these commands:

| Command | Description |
| --- | --- |
| `draw` | Roll one fruit from the weighted pool |
| `inventory` | Show the fruits currently stored |
| `pool` | Show all fruits and their weights |
| `buy-dlc <code>` | Increase duplicate storage by 1 when `<code>` matches any non-empty line in `DLC/p1fs.txt` |
| `help` | Show available commands |
| `exit` or `quit` | Leave the demo |

## Project Structure

```text
src/
  data.rs   Default fruit pool and DLC code helpers
  gacha.rs  Core gacha, inventory, rarity, and tests
  main.rs   Interactive command-line interface
```

## Notes

Fruit weights are defined in `src/data.rs`. A fruit with weight `0.0` can be rolled if you have found a glitch, while very small positive weights make a fruit extremely rare.
