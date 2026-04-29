use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::codex::{self, BookPage};
use crate::run_state::RunState;

pub fn handle_key(state: &mut RunState, key: KeyEvent) -> Option<bool> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    clamp_active_cursor(state);
    match key.code {
        KeyCode::Esc | KeyCode::Char('b') => Some(true),
        KeyCode::Left | KeyCode::Char('a') => {
            state.book_page = state.book_page.previous();
            clamp_active_cursor(state);
            Some(false)
        }
        KeyCode::Right | KeyCode::Char('d') => {
            state.book_page = state.book_page.next();
            clamp_active_cursor(state);
            Some(false)
        }
        KeyCode::Up | KeyCode::Char('w') => {
            move_cursor(state, -1);
            Some(false)
        }
        KeyCode::Down | KeyCode::Char('x') => {
            move_cursor(state, 1);
            Some(false)
        }
        _ => None,
    }
}

fn move_cursor(state: &mut RunState, delta: isize) {
    let len = codex::page_len(state.book_page);
    if len == 0 {
        *active_cursor_mut(state) = 0;
        return;
    }

    let cursor = active_cursor_mut(state);
    let next = (*cursor as isize + delta).rem_euclid(len as isize) as usize;
    *cursor = next;
}

fn clamp_active_cursor(state: &mut RunState) {
    let len = codex::page_len(state.book_page);
    let cursor = active_cursor_mut(state);
    if len == 0 {
        *cursor = 0;
    } else if *cursor >= len {
        *cursor = len - 1;
    }
}

fn active_cursor_mut(state: &mut RunState) -> &mut usize {
    match state.book_page {
        BookPage::Mob => &mut state.book_mob_cursor,
        BookPage::Item => &mut state.book_item_cursor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};
    use hecs::World;
    use rand::SeedableRng;
    use rand_pcg::Pcg64Mcg;

    use crate::codex::CodexProfile;
    use crate::map::Map;
    use crate::run_state::UiMode;
    use crate::ui::MessageLog;

    fn test_state() -> RunState {
        RunState {
            seed: 1,
            depth: 1,
            map: Map::test_arena(20, 12),
            world: World::new(),
            log: MessageLog::new(),
            rng: Pcg64Mcg::seed_from_u64(1),
            mode: UiMode::Book,
            finalized: false,
            inventory_cursor: 0,
            codex: CodexProfile::default(),
            book_page: BookPage::Mob,
            book_mob_cursor: 0,
            book_item_cursor: 0,
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn switches_pages_and_preserves_each_cursor() {
        let mut state = test_state();
        state.book_mob_cursor = 3;

        handle_key(&mut state, key(KeyCode::Right));
        state.book_item_cursor = 5;
        handle_key(&mut state, key(KeyCode::Left));

        assert_eq!(state.book_page, BookPage::Mob);
        assert_eq!(state.book_mob_cursor, 3);
        assert_eq!(state.book_item_cursor, 5);
    }

    #[test]
    fn cursor_wraps_on_active_page() {
        let mut state = test_state();

        handle_key(&mut state, key(KeyCode::Up));
        assert_eq!(state.book_mob_cursor, codex::page_len(BookPage::Mob) - 1);

        handle_key(&mut state, key(KeyCode::Down));
        assert_eq!(state.book_mob_cursor, 0);
    }

    #[test]
    fn esc_and_b_close_the_book() {
        let mut state = test_state();
        assert_eq!(handle_key(&mut state, key(KeyCode::Esc)), Some(true));
        assert_eq!(handle_key(&mut state, key(KeyCode::Char('b'))), Some(true));
    }
}
