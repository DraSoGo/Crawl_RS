//! crawl-rs entry point.
//!
//! Architecture overview:
//! - `cli`        — argument parsing + headless `--dump` mode.
//! - `app`        — top-level screen state machine + event loop.
//! - `run_state`  — per-run state and the helpers that mutate it.
//! - `draw`       — render pipeline for the active run.
//! - `ecs/*`      — components and systems (input, movement, combat, AI,
//!                   FOV, pickup, inventory, render).
//! - `map/*`      — tile map, BSP generation, shadowcasting FOV.
//! - `game/*`     — turn scheduler and level lifecycle.
//! - `save/*`     — bincode save snapshot + high-score table.
//! - `ui/*`       — terminal back-buffer, message log, menus, title screen.
//! - `term`       — raw-mode RAII guard + panic-safe terminal restore.

mod app;
mod cli;
mod data;
mod draw;
mod ecs;
mod game;
mod map;
mod run_state;
mod save;
mod term;
mod ui;

use anyhow::Result;

use crate::cli::{dump_maps, parse_args};
use crate::term::{install_panic_hook, TermGuard};

fn main() -> Result<()> {
    let opts = parse_args()?;
    if opts.dump {
        return dump_maps(opts.seed, opts.dump_count, opts.dump_width, opts.dump_height);
    }
    install_panic_hook();
    let guard = TermGuard::enter()?;
    let result = app::drive(opts);
    guard.leave()?;
    result
}
