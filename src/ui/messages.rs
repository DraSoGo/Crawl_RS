//! Rolling message log shown beneath the map.
//!
//! Stores the last `MAX_MESSAGES` lines as `(text, severity)` pairs; the
//! renderer pulls the tail for display. Severity drives colour so the player
//! can scan the log for the events they care about (combat, loot, status).

use std::collections::VecDeque;

use crossterm::style::Color;
use serde::{Deserialize, Serialize};

const MAX_MESSAGES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Combat,
    #[allow(dead_code)]
    Loot,
    Status,
    Danger,
}

impl Severity {
    pub fn color(self) -> Color {
        match self {
            Severity::Info => Color::Grey,
            Severity::Combat => Color::Yellow,
            Severity::Loot => Color::Cyan,
            Severity::Status => Color::Green,
            Severity::Danger => Color::Red,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Message {
    pub text: String,
    pub severity: Severity,
}

#[derive(Clone, Debug, Default)]
pub struct MessageLog {
    entries: VecDeque<Message>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, text: impl Into<String>, severity: Severity) {
        self.entries.push_back(Message {
            text: text.into(),
            severity,
        });
        while self.entries.len() > MAX_MESSAGES {
            self.entries.pop_front();
        }
    }

    pub fn info(&mut self, text: impl Into<String>) {
        self.push(text, Severity::Info);
    }

    pub fn combat(&mut self, text: impl Into<String>) {
        self.push(text, Severity::Combat);
    }

    #[allow(dead_code)]
    pub fn loot(&mut self, text: impl Into<String>) {
        self.push(text, Severity::Loot);
    }

    pub fn status(&mut self, text: impl Into<String>) {
        self.push(text, Severity::Status);
    }

    pub fn danger(&mut self, text: impl Into<String>) {
        self.push(text, Severity::Danger);
    }

    /// Newest-last slice of the most recent `n` messages.
    pub fn tail(&self, n: usize) -> Vec<&Message> {
        let len = self.entries.len();
        let start = len.saturating_sub(n);
        self.entries.iter().skip(start).collect()
    }
}
