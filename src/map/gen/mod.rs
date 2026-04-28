//! Map generators. Phase 4 ships BSP only; Phase 9 will add more themes.

pub mod bsp;

pub use bsp::{generate as bsp_generate, BspConfig, Dungeon};
