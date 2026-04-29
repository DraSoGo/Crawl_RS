//! Per-run state: world, map, scheduler RNG, log, UI mode. Plus the helpers
//! that mutate it (advance one player turn, descend stairs, save, finalize).

use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::style::Color;
use hecs::World;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

use crate::ecs::components::{
    BlocksTile, Energy, Equipment, FieldOfView, Inventory, Name, Player, Position,
    Progression, Renderable, Stats,
};
use crate::ecs::systems::{combat, fov as fov_sys, movement, pickup};
use crate::game::level::{self, FINAL_DEPTH};
use crate::game::turn::{self, TURN_THRESHOLD};
use crate::map::{Map, Tile};
use crate::save::{self, scores::{self, ScoreEntry}};
use crate::ui::{Buffer, MessageLog};

pub const PLAYER_LAYER: u8 = 200;
pub const HUD_ROWS: u16 = 1;
pub const LOG_ROWS: u16 = 5;
pub const RESERVED_ROWS: u16 = HUD_ROWS + LOG_ROWS;
pub const PLAYER_FOV_RADIUS: i32 = 8;
const PLAYER_BASE_HP: i32 = 20;
const PLAYER_BASE_ATTACK: i32 = 4;
const PLAYER_BASE_DEFENSE: i32 = 1;
const PLAYER_BASE_SPEED: i32 = 10;
const DESCENT_HEAL: i32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMode {
    Playing,
    Inventory,
    GameOver,
    Victory,
}

pub struct RunState {
    pub seed: u64,
    pub depth: u32,
    pub map: Map,
    pub world: World,
    pub log: MessageLog,
    pub rng: Pcg64Mcg,
    pub mode: UiMode,
    pub finalized: bool,
    /// Highlighted slot in the inventory screen. Persisted across opens so
    /// the cursor lands where it was last left.
    pub inventory_cursor: usize,
}

pub fn level_dims(buffer: &Buffer) -> (i32, i32) {
    let w = (buffer.width() as i32).max(20);
    let h = ((buffer.height().saturating_sub(RESERVED_ROWS)) as i32).max(12);
    (w, h)
}

pub fn start_new_run(seed: u64, buffer: &Buffer) -> RunState {
    let depth = 1u32;
    let (w, h) = level_dims(buffer);
    let mut world = World::new();
    spawn_player_skeleton(&mut world);
    let map = level::build_level(&mut world, seed, depth, w, h);
    let mut state = RunState {
        seed,
        depth,
        map,
        world,
        log: MessageLog::new(),
        rng: Pcg64Mcg::seed_from_u64(seed ^ 0xA17E_CAFE_F00D_BEEF),
        mode: UiMode::Playing,
        finalized: false,
        inventory_cursor: 0,
    };
    fov_sys::update(&mut state.world, &state.map);
    state
        .log
        .info(format!("you enter the dungeon (seed {:016x}).", state.seed));
    save_run(&state);
    state
}

fn spawn_player_skeleton(world: &mut World) {
    world.spawn((
        Position::new(0, 0),
        Renderable::new('@', Color::Yellow, Color::Reset, PLAYER_LAYER),
        Player,
        BlocksTile,
        Stats::new(
            PLAYER_BASE_HP,
            PLAYER_BASE_ATTACK,
            PLAYER_BASE_DEFENSE,
            PLAYER_BASE_SPEED,
        ),
        Energy::new(TURN_THRESHOLD),
        Progression::default(),
        Inventory::default(),
        Equipment::default(),
        Name("you".to_string()),
        FieldOfView::new(PLAYER_FOV_RADIUS, 1, 1),
    ));
}

pub fn advance_player_turn(state: &mut RunState) {
    movement::apply(&mut state.world, &state.map);
    combat::resolve(&mut state.world, &mut state.log, &mut state.rng);
    combat::reap(&mut state.world);
    let outcome = pickup::run(&mut state.world, &mut state.log);
    if outcome.picked_amulet {
        state.mode = UiMode::Victory;
        return;
    }
    if combat::player_dead(&state.world) {
        state.mode = UiMode::GameOver;
        return;
    }
    fov_sys::update(&mut state.world, &state.map);
    turn::spend_player_energy(&mut state.world);
    turn::run_npcs_until_player_turn(
        &mut state.world,
        &state.map,
        &mut state.log,
        &mut state.rng,
    );
    if combat::player_dead(&state.world) {
        state.mode = UiMode::GameOver;
    }
    fov_sys::update(&mut state.world, &state.map);
}

pub fn try_descend(state: &mut RunState, buffer: &Buffer) {
    let pos = match player_position(&state.world) {
        Some(p) => p,
        None => return,
    };
    if state.map.tile(pos.x, pos.y) != Some(Tile::DownStairs) {
        state.log.info("there are no stairs here.");
        return;
    }
    if state.depth >= FINAL_DEPTH {
        state.log.info("you have already reached the bottom.");
        return;
    }
    state.depth += 1;
    level::purge_non_player(&mut state.world);
    let (w, h) = level_dims(buffer);
    state.map = level::build_level(&mut state.world, state.seed, state.depth, w, h);
    heal_player(&mut state.world, DESCENT_HEAL);
    fov_sys::update(&mut state.world, &state.map);
    if state.depth == FINAL_DEPTH {
        state
            .log
            .danger(format!("you reach depth {}.", state.depth));
        state
            .log
            .status("the Amulet of Yendor glints somewhere on this floor.");
    } else {
        state
            .log
            .info(format!("you descend to depth {}.", state.depth));
    }
    save_run(state);
}

pub fn save_or_finalize(state: &mut RunState) {
    match state.mode {
        UiMode::GameOver => finalize(state, false),
        UiMode::Victory => finalize(state, true),
        _ => save_run(state),
    }
}

pub fn save_run(state: &RunState) {
    if matches!(state.mode, UiMode::GameOver | UiMode::Victory) {
        return;
    }
    if let Ok(snap) = save::build_snapshot(
        state.seed,
        state.depth,
        &state.map,
        &state.world,
        &state.log,
    ) {
        let _ = save::save(&snap);
    }
}

pub fn finalize(state: &mut RunState, won: bool) {
    if state.finalized {
        return;
    }
    state.finalized = true;
    let _ = save::delete();
    let mut table = scores::load().unwrap_or_default();
    let xp = player_xp(&state.world).unwrap_or(0);
    let kills = player_kills(&state.world).unwrap_or(0);
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    table.record(ScoreEntry {
        seed: state.seed,
        depth: state.depth,
        xp,
        kills,
        won,
        epoch_seconds: epoch,
    });
    let _ = scores::save(&table);
}

pub fn player_position(world: &World) -> Option<Position> {
    world
        .query::<(&Player, &Position)>()
        .iter()
        .map(|(_, (_, p))| *p)
        .next()
}

pub fn player_hp(world: &World) -> Option<(i32, i32)> {
    world
        .query::<(&Player, &Stats)>()
        .iter()
        .map(|(_, (_, s))| (s.hp, s.max_hp))
        .next()
}

pub fn player_combat(world: &World) -> Option<(i32, i32)> {
    world
        .query::<(&Player, &Stats)>()
        .iter()
        .map(|(_, (_, s))| (s.attack, s.defense))
        .next()
}

pub fn player_xp(world: &World) -> Option<i32> {
    world
        .query::<(&Player, &Progression)>()
        .iter()
        .map(|(_, (_, p))| p.xp)
        .next()
}

pub fn player_kills(world: &World) -> Option<u32> {
    world
        .query::<(&Player, &Progression)>()
        .iter()
        .map(|(_, (_, p))| p.kills)
        .next()
}

fn heal_player(world: &mut World, amount: i32) {
    let entity = world.query::<&Player>().iter().next().map(|(e, _)| e);
    if let Some(entity) = entity {
        if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
            stats.hp = (stats.hp + amount).min(stats.max_hp);
        }
    }
}
