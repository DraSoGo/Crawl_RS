# ⚔️ Crawl RS: Roguelike game in terminal


<p align="center">
  <img src="assets/logo.png" width="500" alt="Crawl RS Logo">
</p>

A classic ASCII roguelike in pure terminal Rust. Procedurally generated
dungeons, turn-based combat, permadeath. Runs over SSH; scales from a phone
shell to a 4K terminal.

---

<p align="center"><img src="assets/example.gif"/></p>

## Install

```
cargo install --path .
```

Pre-built binaries (linux-x64, macos-arm64, windows-x64) attach to each
GitHub release.

## Run

```
crawl-rs                # title screen → new game / continue / quit
crawl-rs --seed 42      # skip menu, start a deterministic run
crawl-rs --dump --count 5 --seed 1
                        # print 5 BSP maps to stdout (no TUI)
~/.cargo/bin/crawl-rs
```

## Controls

| Key                        | Action                        |
|----------------------------|-------------------------------|
| `w a s d` / arrow keys     | move (4-way)                  |
| `q e z x`                  | move diagonally (NW NE SW SE) |
| `.`                        | wait one turn                 |
| `f` or `,`                 | pick up item                  |
| `i`                        | open inventory                |
| `>` (on `>` tile)          | descend stairs                |
| `esc` / `ctrl-c`           | save and quit                 |

`q` is the NW diagonal during play, so quitting is bound to `esc` (or
`ctrl-c`) to avoid clobbering a movement key. On the title screen and the
death/victory screens, `q` still works as quit.

In the inventory screen, press `a` … `z` to use or equip the item in that
slot. Use a potion of healing for HP, a scroll of mapping to reveal the
level, a scroll of teleport to fling yourself, or wear armor / wield weapons
for permanent stat bonuses.

## How it works

- ECS via [`hecs`](https://crates.io/crates/hecs)
- BSP dungeon generation per level (10 levels, increasing density)
- Recursive shadowcasting FOV (8 octants, radius 8) with memory tiles
- Energy-based scheduler (`speed` per tick, act at 100)
- Bump-to-attack combat: `max(1, atk + 1d4 - def)`
- Bincode save (single slot, deleted on death)
- Deterministic given `--seed N` — record this in any bug report

## Win condition

Reach depth 10 and pick up the Amulet of Yendor. Permadeath: you die, the
save file is gone, and your score is recorded in the high-score table.

## Notable seeds

See [`examples/seeds.txt`](examples/seeds.txt).

## Demo

Recording a play session: `asciinema rec demo.cast` then play with
`asciinema play demo.cast`. Drop the recording into the README of any fork
you make.

## License

MIT — see [LICENSE](LICENSE).
