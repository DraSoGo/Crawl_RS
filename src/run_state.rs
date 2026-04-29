//! Per-run state: world, map, scheduler RNG, log, UI mode. Plus the helpers
//! that mutate it (advance one player turn, descend stairs, save, finalize).

use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::style::Color;
use hecs::World;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;

use crate::config;
use crate::codex::{self, BookPage, CodexProfile};
pub use crate::ecs::components::HungerClock;
use crate::ecs::components::{
    BlocksTile, Equipment, FieldOfView, Inventory, Name, Player, Position,
    Progression, Renderable, Stats, StatusEffects,
};
use crate::ecs::systems::{combat, fov as fov_sys, movement, pickup};
use crate::game::level::{self, FINAL_DEPTH};
use crate::game::turn;
use crate::map::{Map, Tile};
use crate::save::{self, scores::{self, ScoreEntry}};
use crate::ui::{Buffer, MessageLog};

pub const PLAYER_LAYER: u8 = config::UI.player_layer;
pub const HUD_ROWS: u16 = config::UI.hud_rows;
pub const LOG_ROWS: u16 = config::UI.log_rows;
pub const RESERVED_ROWS: u16 = HUD_ROWS + LOG_ROWS;
pub const PLAYER_FOV_RADIUS: i32 = config::PLAYER.fov_radius;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMode {
    Playing,
    Inventory,
    Book,
    Status,
    Help,
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
    pub codex: CodexProfile,
    pub book_page: BookPage,
    pub book_mob_cursor: usize,
    pub book_item_cursor: usize,
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
    let codex = load_codex_profile();
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
        codex,
        book_page: BookPage::Mob,
        book_mob_cursor: 0,
        book_item_cursor: 0,
    };
    update_visibility_and_codex(&mut state);
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
            config::PLAYER.base_hp,
            config::PLAYER.base_attack,
            config::PLAYER.base_defense,
            config::PLAYER.base_move,
        ),
        Progression::default(),
        Inventory::default(),
        Equipment::default(),
        StatusEffects::default(),
        HungerClock::new(config::PLAYER.start_satiation),
        Name("you".to_string()),
        FieldOfView::new(PLAYER_FOV_RADIUS, 1, 1),
    ));
}

pub fn advance_player_turn(state: &mut RunState) {
    let _ = movement::apply(&mut state.world, &state.map);
    combat::resolve(&mut state.world, &mut state.log, &mut state.rng);
    combat::reap(&mut state.world);
    update_visibility_and_codex(state);
    let outcome = pickup::run(&mut state.world, &mut state.log);
    if outcome.picked_amulet {
        state.mode = UiMode::Victory;
        return;
    }
    if combat::player_dead(&state.world) {
        state.mode = UiMode::GameOver;
        return;
    }
    turn::run_enemy_turn(
        &mut state.world,
        &state.map,
        &mut state.log,
        &mut state.rng,
    );
    if combat::player_dead(&state.world) {
        state.mode = UiMode::GameOver;
    }
    update_visibility_and_codex(state);
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
    heal_player(&mut state.world, config::PLAYER.descent_heal);
    update_visibility_and_codex(state);
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

pub fn load_codex_profile() -> CodexProfile {
    crate::save::codex::load().unwrap_or_default()
}

pub fn update_visibility_and_codex(state: &mut RunState) {
    fov_sys::update(&mut state.world, &state.map);
    let discoveries = codex::discover_visible_entries(&state.world);
    if codex::apply_discoveries(&mut state.codex, discoveries) {
        let _ = crate::save::codex::save(&state.codex);
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

#[allow(dead_code)]
pub fn player_hunger(world: &World) -> Option<&'static str> {
    use crate::ecs::components::HungerState;
    let player = world.query::<&Player>().iter().next().map(|(e, _)| e)?;
    let h = world.get::<&HungerClock>(player).ok()?;
    Some(match h.state() {
        HungerState::Sated => "sated",
        HungerState::Hungry => "hungry",
        HungerState::Starving => "STARVING",
    })
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

pub fn player_level(world: &World) -> Option<u32> {
    world
        .query::<(&Player, &Progression)>()
        .iter()
        .map(|(_, (_, p))| p.level)
        .next()
}

/// Award XP to whichever entity is the player. Loops while XP exceeds the
/// next-level threshold so a single big award can grant several levels at
/// once. Each level grants +5 max HP (+5 current HP), +1 attack, +1 defense.
pub fn award_xp(world: &mut World, log: &mut MessageLog, amount: i32) {
    if amount <= 0 {
        return;
    }
    let player = match world.query::<&Player>().iter().next().map(|(e, _)| e) {
        Some(e) => e,
        None => return,
    };
    let mut levels_gained = 0u32;
    let mut new_level = 0u32;
    let mut hp_bonus_total = 0;
    if let Ok(mut prog) = world.get::<&mut Progression>(player) {
        prog.xp = prog.xp.saturating_add(amount);
        loop {
            let needed = Progression::xp_for_next(prog.level);
            if prog.xp < needed {
                break;
            }
            prog.xp -= needed;
            prog.level += 1;
            levels_gained += 1;
        }
        new_level = prog.level;
    }
    if levels_gained == 0 {
        return;
    }
    if let Ok(mut stats) = world.get::<&mut Stats>(player) {
        let hp_bump = config::PLAYER.level_up_hp * levels_gained as i32;
        stats.max_hp += hp_bump;
        stats.hp = (stats.hp + hp_bump).min(stats.max_hp);
        stats.attack += config::PLAYER.level_up_attack * levels_gained as i32;
        stats.defense += config::PLAYER.level_up_defense * levels_gained as i32;
        hp_bonus_total = hp_bump;
    }
    let attack_bump = config::PLAYER.level_up_attack * levels_gained as i32;
    let defense_bump = config::PLAYER.level_up_defense * levels_gained as i32;
    log.status(format!(
        "you reach level {new_level}! (+{hp_bonus_total} max hp, +{attack_bump} atk, +{defense_bump} def, +{} pack slots)",
        levels_gained as usize * config::PROGRESSION.inventory_slots_per_level
    ));
}

fn heal_player(world: &mut World, amount: i32) {
    let entity = world.query::<&Player>().iter().next().map(|(e, _)| e);
    if let Some(entity) = entity {
        if let Ok(mut stats) = world.get::<&mut Stats>(entity) {
            stats.hp = (stats.hp + amount).min(stats.max_hp);
        }
    }
}
