use anyhow::{bail, Result};
use std::path::Path;
use std::process::Command;
/// Opens a repository folder without invoking a shell or interpolating its path.
pub fn open_folder(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("folder does not exist")
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe").arg(path).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}
