//! Input system: maps a single key event to an `Action` describing what the
//! game loop should do next. Movement intents are written into the world here
//! so the movement system can consume them in the same turn.

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use hecs::World;

use crate::ecs::components::{MoveIntent, Player, WantsToPickup};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    /// Player did something this turn (movement, pickup, etc.).
    Took,
    /// Player asked to quit the game.
    Quit,
    /// Open the inventory screen — does not consume a turn.
    OpenInventory,
    /// Open the feature book — does not consume a turn.
    OpenBook,
    /// Descend a flight of stairs.
    Descend,
    /// Key was not bound to anything; no turn elapsed.
    None,
}

pub fn handle_key(world: &mut World, key: KeyEvent) -> PlayerAction {
    if key.kind != KeyEventKind::Press {
        return PlayerAction::None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return PlayerAction::Quit;
    }

    match key.code {
        // WASD-style layout. `q` is NW diagonal during play; quit is bound
        // to Esc (and Ctrl-C above) so it doesn't clash with movement.
        KeyCode::Esc => PlayerAction::Quit,
        KeyCode::Up | KeyCode::Char('w') => queue_move(world, 0, -1),
        KeyCode::Down | KeyCode::Char('s') => queue_move(world, 0, 1),
        KeyCode::Left | KeyCode::Char('a') => queue_move(world, -1, 0),
        KeyCode::Right | KeyCode::Char('d') => queue_move(world, 1, 0),
        KeyCode::Char('q') => queue_move(world, -1, -1),
        KeyCode::Char('e') => queue_move(world, 1, -1),
        KeyCode::Char('z') => queue_move(world, -1, 1),
        KeyCode::Char('x') => queue_move(world, 1, 1),
        KeyCode::Char('.') => PlayerAction::Took,
        KeyCode::Char('f') | KeyCode::Char(',') => queue_pickup(world),
        KeyCode::Char('i') => PlayerAction::OpenInventory,
        KeyCode::Char('b') => PlayerAction::OpenBook,
        KeyCode::Char('>') => PlayerAction::Descend,
        _ => PlayerAction::None,
    }
}

fn queue_move(world: &mut World, dx: i32, dy: i32) -> PlayerAction {
    if let Some(entity) = player_entity(world) {
        let _ = world.insert_one(entity, MoveIntent::new(dx, dy));
        PlayerAction::Took
    } else {
        PlayerAction::None
    }
}

fn queue_pickup(world: &mut World) -> PlayerAction {
    if let Some(entity) = player_entity(world) {
        let _ = world.insert_one(entity, WantsToPickup);
        PlayerAction::Took
    } else {
        PlayerAction::None
    }
}

fn player_entity(world: &World) -> Option<hecs::Entity> {
    world.query::<&Player>().iter().next().map(|(e, _)| e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};

    #[test]
    fn b_opens_the_book() {
        let mut world = World::new();
        let action = handle_key(&mut world, KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE));
        assert_eq!(action, PlayerAction::OpenBook);
    }
}
