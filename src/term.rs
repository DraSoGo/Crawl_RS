//! Terminal lifecycle: raw mode, alternate screen, panic-safe restore.

use std::io::{self, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor,
    event::DisableMouseCapture,
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

/// RAII guard. Drop restores the terminal even on panic so the user's shell
/// stays usable. We also install a panic hook in `install_panic_hook` for
/// belt-and-braces safety: drop alone is not enough if the panic aborts before
/// unwinding (e.g. if the user has set `panic = "abort"` in a profile).
pub struct TermGuard {
    active: bool,
}

impl TermGuard {
    pub fn enter() -> Result<Self> {
        enable_raw_mode().context("enable raw mode")?;
        let mut out = io::stdout();
        execute!(out, EnterAlternateScreen, cursor::Hide)
            .context("enter alternate screen")?;
        Ok(Self { active: true })
    }

    pub fn leave(mut self) -> Result<()> {
        self.restore()
    }

    fn restore(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }
        self.active = false;
        let mut out = io::stdout();
        // Best-effort: continue cleanup even if one step fails.
        let _ = execute!(out, cursor::Show, LeaveAlternateScreen, DisableMouseCapture);
        let _ = disable_raw_mode();
        let _ = out.flush();
        Ok(())
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Install a panic hook that restores the terminal before the default hook
/// prints the panic message — otherwise the message lands inside the alternate
/// screen and is wiped when the screen is left.
pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut out = io::stdout();
        let _ = execute!(out, cursor::Show, LeaveAlternateScreen, DisableMouseCapture);
        let _ = disable_raw_mode();
        let _ = out.flush();
        default(info);
    }));
}
