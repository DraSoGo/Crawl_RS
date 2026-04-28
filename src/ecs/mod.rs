//! ECS layer. Wraps `hecs::World` plus components and systems used by the
//! game loop. The world is the single source of truth for game state.

pub mod components;
pub mod systems;
