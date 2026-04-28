//! Static game-data tables (mobs, items, prefabs). Kept separate from ECS
//! and systems so balance changes don't churn the engine code.

pub mod items;
pub mod mobs;
