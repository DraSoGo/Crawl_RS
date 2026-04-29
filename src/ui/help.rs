use crate::run_state::RunState;

use super::Buffer;

const HELP_ROWS: &[(&str, &str)] = &[
    ("w a s d", "move cardinal"),
    ("q e z x", "move diagonal"),
    (".", "wait one turn"),
    ("f ,", "pick up item"),
    ("i", "open inventory"),
    ("b", "open book"),
    ("k", "open status"),
    ("h", "open help"),
    (">", "descend stairs"),
    ("esc", "save and quit"),
    ("inventory: up/down", "select item"),
    ("inventory: f", "use or equip"),
    ("inventory: g", "sell item"),
    ("inventory: esc / i", "close inventory"),
    ("book: left/right", "switch page"),
    ("book: up/down", "select entry"),
    ("book: b / esc", "close book"),
    ("status: k / esc / enter", "close status"),
    ("help: h / esc / enter", "close help"),
];

pub fn draw_help(_state: &RunState, buffer: &mut Buffer) {
    let body = help_lines();
    super::layout::draw_panel(
        buffer,
        super::layout::PanelBlock::new(
            "Help",
            &body,
            "press h / esc / enter to close",
        ),
    );
}

fn help_lines() -> Vec<String> {
    let key_width = HELP_ROWS
        .iter()
        .map(|(key, _)| key.chars().count())
        .max()
        .unwrap_or(0);
    let mut lines = Vec::with_capacity(HELP_ROWS.len() + 2);
    lines.push(format!("{:<key_width$}   {}", "Key", "Action"));
    lines.push(format!("{:-<key_width$}   {:-<6}", "", ""));
    for (key, action) in HELP_ROWS {
        lines.push(format!("{:<key_width$}   {}", key, action));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_contains_key_action_header() {
        let lines = help_lines();
        assert!(lines.iter().any(|line| line.contains("Key")));
        assert!(lines.iter().any(|line| line.contains("Action")));
        assert!(lines.iter().any(|line| line.contains("inventory: g")));
    }
}
