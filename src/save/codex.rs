use std::fs;
use std::io;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;

use crate::codex::CodexProfile;

const CODEX_FILENAME: &str = "codex.bin";

fn codex_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("dev", "crawl-rs", "crawl-rs")
        .ok_or_else(|| anyhow!("no home directory available for codex"))?;
    let dir = dirs.data_dir().to_path_buf();
    fs::create_dir_all(&dir).context("create codex dir")?;
    Ok(dir.join(CODEX_FILENAME))
}

pub fn load() -> Result<CodexProfile> {
    let path = codex_path()?;
    match fs::read(&path) {
        Ok(bytes) => bincode::deserialize::<CodexProfile>(&bytes)
            .context("deserialize codex"),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Err(anyhow!("no codex file at {}", path.display()))
        }
        Err(e) => Err(anyhow::Error::from(e)).context("read codex"),
    }
}

pub fn save(profile: &CodexProfile) -> Result<()> {
    let path = codex_path()?;
    let bytes = bincode::serialize(profile).context("serialize codex")?;
    fs::write(&path, bytes).context("write codex")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_profile_round_trips_through_bincode() {
        let mut profile = CodexProfile::default();
        profile.discovered_mobs.insert("rat".to_string());
        profile
            .discovered_items
            .insert("potion of healing".to_string());

        let bytes = bincode::serialize(&profile).expect("serialize codex");
        let restored: CodexProfile =
            bincode::deserialize(&bytes).expect("deserialize codex");

        assert!(restored.discovered_mobs.contains("rat"));
        assert!(restored.discovered_items.contains("potion of healing"));
    }
}
