//! Top-level screen state machine + event loop.

use std::io::{self, Write};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

use crate::cli::CliOpts;
use crate::draw::draw_run;
use crate::ecs::systems::{
    fov as fov_sys,
    input::{handle_key, PlayerAction},
    inventory as inventory_sys,
};
use crate::game::turn;
use crate::run_state::{
    advance_player_turn, save_or_finalize, save_run, start_new_run, try_descend, RunState,
    UiMode,
};
use crate::save;
use crate::ui::title::{MenuChoice, MenuState};
use crate::ui::{title, Buffer};

const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub enum Screen {
    MainMenu(MenuState),
    Run(RunState),
}

pub fn drive(opts: CliOpts) -> Result<()> {
    let (cols, rows) = terminal::size().context("query terminal size")?;
    let mut buffer = Buffer::new(cols.max(1), rows.max(1));
    let mut stdout = io::stdout();

    let mut screen = if let Some(seed) = opts.seed {
        Screen::Run(start_new_run(seed, &buffer))
    } else {
        Screen::MainMenu(MenuState::new(save::exists()))
    };
    let mut needs_redraw = true;

    loop {
        if needs_redraw {
            buffer.clear();
            match &screen {
                Screen::MainMenu(menu) => title::draw(&mut buffer, menu),
                Screen::Run(state) => draw_run(&mut buffer, state),
            }
            buffer.flush(&mut stdout)?;
            stdout.flush().context("flush stdout")?;
            needs_redraw = false;
        }

        if !event::poll(POLL_INTERVAL).context("poll events")? {
            continue;
        }
        match event::read().context("read event")? {
            Event::Key(key) => match &mut screen {
                Screen::MainMenu(_) => match handle_menu_key(&mut screen, &buffer, key) {
                    Some(MenuOutcome::Quit) => return Ok(()),
                    Some(_) => needs_redraw = true,
                    None => {}
                },
                Screen::Run(_) => match handle_run_key(&mut screen, &buffer, key)? {
                    RunOutcome::Quit => return Ok(()),
                    RunOutcome::ToMenu => {
                        screen = Screen::MainMenu(MenuState::new(save::exists()));
                        needs_redraw = true;
                    }
                    RunOutcome::Redraw => needs_redraw = true,
                    RunOutcome::None => {}
                },
            },
            Event::Resize(w, h) => {
                buffer.resize(w.max(1), h.max(1));
                if let Screen::Run(state) = &mut screen {
                    let seed = state.seed;
                    *state = start_new_run(seed, &buffer);
                }
                needs_redraw = true;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum MenuOutcome {
    NewGame,
    Continue,
    Quit,
}

fn handle_menu_key(
    screen: &mut Screen,
    buffer: &Buffer,
    key: KeyEvent,
) -> Option<MenuOutcome> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Some(MenuOutcome::Quit);
    }
    let menu = match screen {
        Screen::MainMenu(m) => m,
        _ => return None,
    };
    let choice = match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            menu.move_up();
            return Some(MenuOutcome::NewGame);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            menu.move_down();
            return Some(MenuOutcome::NewGame);
        }
        KeyCode::Char('n') | KeyCode::Char('N') => MenuChoice::NewGame,
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if menu
                .items
                .iter()
                .any(|(c, en)| *c == MenuChoice::Continue && *en)
            {
                MenuChoice::Continue
            } else {
                return None;
            }
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => MenuChoice::Quit,
        KeyCode::Enter => menu.current(),
        _ => return None,
    };
    apply_menu_choice(screen, buffer, choice)
}

fn apply_menu_choice(
    screen: &mut Screen,
    buffer: &Buffer,
    choice: MenuChoice,
) -> Option<MenuOutcome> {
    match choice {
        MenuChoice::NewGame => {
            let _ = save::delete();
            let seed = crate::cli::seed_from_clock();
            *screen = Screen::Run(start_new_run(seed, buffer));
            Some(MenuOutcome::NewGame)
        }
        MenuChoice::Continue => match save::load() {
            Ok(snap) => {
                let restore = save::restore(snap);
                let mut state = RunState {
                    seed: restore.seed,
                    depth: restore.depth,
                    map: restore.map,
                    world: restore.world,
                    log: restore.log,
                    rng: Pcg64Mcg::seed_from_u64(
                        restore.seed ^ 0xA17E_CAFE_F00D_BEEF,
                    ),
                    mode: UiMode::Playing,
                    finalized: false,
                    inventory_cursor: 0,
                };
                fov_sys::update(&mut state.world, &state.map);
                state.log.info("you continue your descent.");
                *screen = Screen::Run(state);
                Some(MenuOutcome::Continue)
            }
            Err(_) => Some(MenuOutcome::NewGame),
        },
        MenuChoice::Quit => Some(MenuOutcome::Quit),
    }
}

#[derive(Clone, Copy, Debug)]
enum RunOutcome {
    None,
    Redraw,
    ToMenu,
    Quit,
}

fn handle_run_key(
    screen: &mut Screen,
    buffer: &Buffer,
    key: KeyEvent,
) -> Result<RunOutcome> {
    let state = match screen {
        Screen::Run(s) => s,
        _ => return Ok(RunOutcome::None),
    };
    if key.kind != KeyEventKind::Press {
        return Ok(RunOutcome::None);
    }
    match state.mode {
        UiMode::GameOver | UiMode::Victory => {
            if matches!(
                key.code,
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter
            ) {
                return Ok(RunOutcome::ToMenu);
            }
            Ok(RunOutcome::None)
        }
        UiMode::Inventory => {
            if let Some(closing) = handle_inventory_key(state, key) {
                if closing && state.mode == UiMode::Inventory {
                    state.mode = UiMode::Playing;
                }
                Ok(RunOutcome::Redraw)
            } else {
                Ok(RunOutcome::None)
            }
        }
        UiMode::Playing => match handle_key(&mut state.world, key) {
            PlayerAction::Quit => {
                save_run(state);
                Ok(RunOutcome::Quit)
            }
            PlayerAction::OpenInventory => {
                state.mode = UiMode::Inventory;
                Ok(RunOutcome::Redraw)
            }
            PlayerAction::Descend => {
                try_descend(state, buffer);
                Ok(RunOutcome::Redraw)
            }
            PlayerAction::Took => {
                advance_player_turn(state);
                save_or_finalize(state);
                Ok(RunOutcome::Redraw)
            }
            PlayerAction::None => Ok(RunOutcome::None),
        },
    }
}

fn handle_inventory_key(state: &mut RunState, key: KeyEvent) -> Option<bool> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    let inv_len = inventory_len(&state.world);
    if state.inventory_cursor >= inv_len {
        state.inventory_cursor = inv_len.saturating_sub(1);
    }
    match key.code {
        KeyCode::Esc | KeyCode::Char('i') => Some(true),
        KeyCode::Up | KeyCode::Char('w') => {
            if inv_len > 0 {
                state.inventory_cursor = (state.inventory_cursor + inv_len - 1) % inv_len;
            }
            Some(false)
        }
        KeyCode::Down | KeyCode::Char('s') => {
            if inv_len > 0 {
                state.inventory_cursor = (state.inventory_cursor + 1) % inv_len;
            }
            Some(false)
        }
        KeyCode::Char('f') | KeyCode::Enter => {
            if inv_len == 0 {
                return Some(false);
            }
            let index = state.inventory_cursor;
            let used = inventory_sys::use_index(
                &mut state.world,
                &mut state.map,
                &mut state.log,
                &mut state.rng,
                index,
            );
            if used {
                let new_len = inventory_len(&state.world);
                if state.inventory_cursor >= new_len {
                    state.inventory_cursor = new_len.saturating_sub(1);
                }
                fov_sys::update(&mut state.world, &state.map);
                turn::spend_player_energy(&mut state.world);
                turn::run_npcs_until_player_turn(
                    &mut state.world,
                    &state.map,
                    &mut state.log,
                    &mut state.rng,
                );
                if crate::ecs::systems::combat::player_dead(&state.world) {
                    state.mode = UiMode::GameOver;
                    return Some(true);
                }
                fov_sys::update(&mut state.world, &state.map);
                save_or_finalize(state);
                Some(true)
            } else {
                Some(false)
            }
        }
        _ => None,
    }
}

fn inventory_len(world: &hecs::World) -> usize {
    use crate::ecs::components::{Inventory, Player};
    world
        .query::<(&Player, &Inventory)>()
        .iter()
        .map(|(_, (_, inv))| inv.items.len())
        .next()
        .unwrap_or(0)
}
