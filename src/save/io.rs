//! Filesystem I/O for the single save slot.

use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;

use crate::save::types::{SaveSnapshot, SAVE_FILENAME, SAVE_VERSION};

pub fn save_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "crawl-rs", "crawl-rs")
        .ok_or_else(|| anyhow!("no home directory available for save file"))?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir).context("create save dir")?;
    Ok(dir.join(SAVE_FILENAME))
}

pub fn exists() -> bool {
    save_path().ok().map(|p| p.exists()).unwrap_or(false)
}

pub fn delete() -> Result<()> {
    let path = save_path()?;
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    }
    Ok(())
}

pub fn save(snapshot: &SaveSnapshot) -> Result<()> {
    let path = save_path()?;
    let bytes = bincode::serialize(snapshot).context("serialize save")?;
    fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn load() -> Result<SaveSnapshot> {
    let path = save_path()?;
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(anyhow!("no save file at {}", path.display()));
        }
        Err(e) => return Err(e).with_context(|| format!("read {}", path.display())),
    };
    let snap: SaveSnapshot = bincode::deserialize(&bytes).context("deserialize save")?;
    if snap.version != SAVE_VERSION {
        return Err(anyhow!(
            "save format version {} not supported (expected {})",
            snap.version,
            SAVE_VERSION
        ));
    }
    Ok(snap)
}
