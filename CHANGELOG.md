# Changelog

All notable changes to crawl-rs are recorded here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) loosely; semver.

## [Unreleased]

## [0.2.0] - 2026-04-28

Extended the dungeon from 10 floors to 20 and added a new late-game
content curve to support the longer run.

### Added
- Deep-floor mob roster for floors 11-20: `crypt knight`, `ash hound`,
  `gargoyle`, `vampire`, `warlock`, `basilisk`, `nightgaunt`,
  `bone colossus`, `void priest`, and `ancient drake`.
- Deep-floor item roster: `potion of giant strength`, `potion of far sight`,
  `potion of fortitude`, `scroll of chain lightning`,
  `scroll of greater fear`, `scroll of legion`, `runed greatsword`,
  `obsidian blade`, `gothic plate`, `dragon scale armor`,
  `ring of regen`, and `wand of storms`.
- New consumable behaviors for chained ranged damage, stronger fear,
  larger allied summons, and multi-target wand zaps.

### Changed
- Final dungeon depth increased from 10 to 20 floors.
- Depth-based mob HP/attack scaling retuned for a 20-floor game so late
  floors remain difficult without runaway stat inflation.
- Existing late-game mobs redistributed deeper into the dungeon to make
  the new floor range distinct from the original 10-floor curve.
- Mob and item selection now bias toward more recently unlocked depth
  tiers so deep floors surface late-game content more reliably.
- README and project guidance updated to document the 20-floor run.

### Fixed
- Save schema bumped to `SAVE_VERSION = 5` so the new item variants are
  versioned correctly instead of loading through the old format.
- Summon scroll placement now skips blocked tiles instead of forcing new
  allies onto occupied cells.

### Tests
- Full `cargo test` suite passes after the content and balance expansion
  (`49` tests).

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
