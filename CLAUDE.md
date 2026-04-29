# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

All commands run from this crate's directory (`crawl-rs/`).

```sh
cargo build                                # dev build
cargo build --release                      # release build (used for cross-platform binaries)
cargo test                                 # full test suite (23 tests, expected to pass)
cargo test bsp::                           # filter: run only tests under map::gen::bsp
cargo test snapshot_round_trips_through_bincode -- --exact   # one specific test by full path
cargo test -- --nocapture                  # show stdout from passing tests

cargo run -- --seed 42                     # skip menu, deterministic run
cargo run -- --dump --count 5 --seed 1 --width 60 --height 22   # headless ASCII dump (no TUI)

cargo install --path . --force             # install ~/.cargo/bin/crawl-rs
```

There is no `rustup` on the development machine, so `cargo clippy` and `cargo fmt` are not runnable locally; they run in CI under `.github/workflows/ci.yml`.

`cargo install` re-resolves dependencies (ignores `Cargo.lock`). Direct version pins for `indexmap` and `hashbrown` in `Cargo.toml` keep MSRV 1.75 building — do not remove them without bumping `rust-version`.

## Architecture

### Crate-level shape

Single binary. Source is split so no file exceeds ~400 lines (split a module if you push past). Top-level modules are wired in `src/main.rs` and have a stable role:

- `cli` — argument parsing + headless `--dump` mode.
- `app` — outer screen state machine (`MainMenu` ↔ `Run`) and the keyboard-event dispatch.
- `run_state` — per-run state (`RunState`) and helpers that mutate it (`advance_player_turn`, `try_descend`, `save_run`, `award_xp`, `finalize`).
- `draw` — render pipeline for the `Run` screen.
- `ecs` — components and systems (input, movement, combat, AI, FOV, pickup, inventory, render, status).
- `map` — tile map, BSP generator, recursive shadowcasting FOV.
- `game` — turn scheduler and level-lifecycle helpers.
- `save` — bincode snapshot pipeline (build / restore / IO / scores).
- `ui` — terminal back-buffer, message log, modal menus, title screen.
- `term` — RAII guard that enables raw mode + alternate screen and a panic hook that always restores them.

### Screen state machine

`app::drive(opts)` is the event loop. It owns one `Screen` enum at a time:

- `Screen::MainMenu(MenuState)` draws the title screen.
- `Screen::Run(RunState)` drives gameplay.

`RunState` itself carries a `UiMode` sub-state (`Playing` | `Inventory` | `GameOver` | `Victory`). The inventory and end screens are rendered as overlays — there is no separate event loop for them.

Resize events rebuild the current run from the same seed (permadeath + fresh map size). `--seed N` skips the menu entirely and starts a deterministic run.

### Turn scheduler

`game::turn::run_npcs_until_player_turn` is the engine of NPC turns. Every iteration:

1. `status::tick` runs DoTs, regen, buff timers, summoner rolls, then `check_deaths` (clamps HP to 0 and inserts `Dead`).
2. `tick_energy` adds each entity's `Stats::speed` to its `Energy`.
3. Each ready (≥`TURN_THRESHOLD = 100`) NPC runs `ai::plan` → `movement::apply` → `combat::resolve` → `combat::reap`.
4. `spend_energy` drains the actor's `Energy` to **≤ 0** (intentional cap — fast mobs do not double-act in one player turn).
5. Loop until the player is ready (`Energy ≥ 100`).

This means every mob acts at most once per player turn; slow mobs (speed < player) skip turns occasionally.

### ECS conventions

Components live in `ecs::components`. They are plain data — behaviour is in `ecs::systems`. Some patterns that matter when extending:

- **Intent components** (`MoveIntent`, `WantsToAttack`, `WantsToPickup`) are inserted by input/AI and consumed within the same turn.
- **`Dead`** is a marker that delays despawn. `combat::reap` only removes mobs; the player is left in place with `Dead` so the death screen can read final stats.
- **`Faction`** distinguishes `PlayerAlly` (summoned mobs) from `Hostile`. `ai::enemy_with_entity` uses it to pick targets.
- **`StatusEffects`** is a flat struct of timers (poison, paralysis, fear, speed/attack/vision buffs, regen, light, invisible). When you add a new timer, also decrement it in `status::tick_status_effects` and undo the bonus when it hits 0.
- **`Inventory.items`** stores `Entity` references to item entities held by the world. Items lose their `Position` on pickup. Crucial: `level::purge_non_player` filters by `Position`, **not** absence of `Player`, so inventory items survive level transitions. Don't change that filter without re-checking save/load.

### Map and FOV

`Map` is a tile grid. BSP (`map::gen::bsp`) returns a `Dungeon` with rooms + start + stairs. Player starts on `<` (UpStairs); descent moves to a fresh map with `level::level_seed(run_seed, depth)` (SplitMix mix → distinct seed per depth, deterministic per `(run_seed, depth)`).

FOV uses recursive shadowcasting — see `map::fov`. Iterate dx in `-row..=0` (steep → shallow); reversing the order breaks the unblock/reblock logic. Don't optimise this loop without tests.

`FieldOfView` carries radius + visibility bitmaps + `dirty`. Anything that moves the viewer or changes the map sets `dirty`; the FOV system recomputes only when set.

### Save format

`save::SaveSnapshot` is a flat projection — hecs's `World` is not directly serialisable so we walk the player + mobs + ground items + amulet manually.

- `SAVE_VERSION` (in `save::types`) is bumped any time a serialised struct gains/loses fields. Old saves fail to load with an error — that is intentional permadeath behaviour.
- Adding a field to `Stats`, `Progression`, `Equipment`, `StatusEffects`, `HungerClock`, `ItemKind`, or any of the `*Snapshot` structs requires bumping `SAVE_VERSION`.
- The save file path is `directories::ProjectDirs::from("dev", "crawl-rs", "crawl-rs").data_dir()/save.bin`. The high-score table sits next to it as `scores.bin`.
- `save::delete()` is called on player death and on victory ("permadeath").

### Determinism

Every random source is seeded from the run's `u64`. The HUD, save file, and death/victory screens all show the seed in hex so a player can replay the same level. The generator uses `Pcg64Mcg::seed_from_u64` (SplitMix-based mixing) — `Pcg64Mcg::new(seed as u128)` is **not** equivalent for small seeds and produces colliding streams.

### Disabled subsystems

The hunger clock (`HungerClock`, `tick_hunger`) is wired into the data model and save format but **not invoked** by `status::tick`. Re-enabling it requires both restoring the call and ensuring `check_deaths` runs after starvation damage. The `tick_hunger` function is `#[allow(dead_code)]` — leave it there as a reference implementation.

The amulet "teleport control" effect, the `Identify` scroll, demon AoE breath, and the dragon's `Flying` flag are placeholder stubs (no behaviour) — see comments in `data::mobs` and `ecs::systems::inventory::effects`.

### Cross-cutting constraints

- No `unwrap()` outside tests. Use `anyhow::Result` + `?`.
- No file longer than ~400 lines. Split a module if it grows past — see how `ecs::systems::inventory` was split into `mod` / `effects` / `equip` / `consume` / `sell` for the pattern.
- The custom panic hook in `term::install_panic_hook` is what keeps the user's terminal usable after a crash. Do not replace `std::panic::set_hook` without preserving the leave-alternate-screen + disable-raw-mode + cursor-show steps.
- The diff-based renderer (`ui::buffer`) only emits cells that differ from the front buffer. Always go through `Buffer::put` / `put_str` / `flush` — emitting raw ANSI elsewhere will desync the front buffer and corrupt the next frame.
