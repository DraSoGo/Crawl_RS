# Changelog

All notable changes to crawl-rs are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) loosely; semver.

## [Unreleased]

## [0.1.0] - 2026-04-28

Initial release. Full Phase 1–11 build per the project plan.

### Added
- Diff-based double-buffered terminal renderer (`crossterm`, no `ratatui`).
- Custom panic hook that restores the terminal on crash.
- ECS via `hecs` with `Position`, `Renderable`, `Player`, `Mob`, `Item`,
  `Stats`, `Energy`, `Inventory`, `Equipment`, `FieldOfView`,
  `Progression`, `Ai`, `WantsToAttack`, `WantsToPickup`, `BlocksTile`.
- BSP dungeon generation with L-corridors and guaranteed connectivity.
- Recursive 8-octant shadowcasting field of view with memory tiles.
- Energy-accumulator turn scheduler (speed per tick, act at 100).
- Hostile chase AI with line-of-sight check.
- Bump-to-attack combat, damage rolls, death cleanup, XP/kill tracking.
- Item types: potions, scrolls (mapping / teleport), weapons, armor;
  inventory + equip/unequip with stat bonuses.
- Multi-level dungeons (10 floors), depth-scaled mob HP/attack and
  spawn density, descent HP regen, Amulet of Yendor on the final floor.
- Permadeath bincode save (single slot, deleted on death/win) with
  full FOV-memory restoration.
- High-score table (`scores.bin`) sorted by win → depth → xp → kills.
- Title-screen ASCII art with new game / continue / quit menu.
- Deterministic seed recorded in HUD, save file, and game-over screen.
- `--seed N`, `--dump`, `--count`, `--width`, `--height` CLI flags.

### Tests
- BSP connectivity (20 seeds), determinism, room bounds.
- FOV: origin visibility, walls block sight, radius limit, memory persists.
- Movement: wall blocking, entity blocking, intent consumption.
- Combat: damage, lethal kill awards XP, player death marks game-over.
- AI: hostile mob chases visible player; LOS sanity.
- Level seed determinism.
- Save snapshot bincode round-trip.

### Known limitations (per Plan.md non-goals)
- No multiplayer, sound, mouse, or tilesets.
- Fixed item table — no procedural item generation.
