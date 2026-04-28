//! High-score table. Append-only JSON-ish lines is overkill for v0.1; we
//! serialise the whole table with bincode each save.

use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

const SCORES_FILENAME: &str = "scores.bin";
const MAX_SCORES: usize = 25;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub seed: u64,
    pub depth: u32,
    pub xp: i32,
    pub kills: u32,
    pub won: bool,
    pub epoch_seconds: u64,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ScoreTable {
    pub entries: Vec<ScoreEntry>,
}

impl ScoreTable {
    pub fn record(&mut self, entry: ScoreEntry) {
        self.entries.push(entry);
        // Sort: wins first, then by depth desc, then by xp desc.
        self.entries.sort_by(|a, b| {
            b.won
                .cmp(&a.won)
                .then(b.depth.cmp(&a.depth))
                .then(b.xp.cmp(&a.xp))
                .then(b.kills.cmp(&a.kills))
        });
        self.entries.truncate(MAX_SCORES);
    }
}

pub fn scores_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "crawl-rs", "crawl-rs")
        .ok_or_else(|| anyhow!("no home directory available for scores"))?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir).context("create scores dir")?;
    Ok(dir.join(SCORES_FILENAME))
}

pub fn load() -> Result<ScoreTable> {
    let path = scores_path()?;
    match fs::read(&path) {
        Ok(bytes) => bincode::deserialize::<ScoreTable>(&bytes)
            .context("deserialize scores")
            .or_else(|_| Ok(ScoreTable::default())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ScoreTable::default()),
        Err(e) => Err(anyhow::Error::from(e)).context("read scores"),
    }
}

pub fn save(table: &ScoreTable) -> Result<()> {
    let path = scores_path()?;
    let bytes = bincode::serialize(table).context("serialize scores")?;
    fs::write(&path, bytes).context("write scores")?;
    Ok(())
}
