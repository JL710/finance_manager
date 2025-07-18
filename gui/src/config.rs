use anyhow::Result;
use std::{io::Write, path::PathBuf};

fn get_config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("finance_manager")
}

pub fn get_config_file(name: &str) -> PathBuf {
    get_config_dir().join(format!("{name}.json"))
}

fn check_create_config_dir() -> Result<()> {
    if !get_config_dir().exists() {
        std::fs::create_dir(get_config_dir())?;
    }

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn read_config(config_name: &'static str) -> Result<Option<serde_json::Value>> {
    async_std::task::spawn_blocking(move || {
        let filepath = get_config_file(config_name);
        if !filepath.exists() {
            return Ok(None);
        }

        let value = serde_json::from_reader(std::fs::File::open(filepath)?)?;

        Ok(Some(value))
    })
    .await
}

#[cfg(target_arch = "wasm32")]
pub async fn read_config(config_name: &str) -> Result<Option<serde_json::Value>> {
    Ok(None)
}

#[cfg(target_arch = "wasm32")]
pub async fn write_config(value: serde_json::Value, config_name: &str) -> Result<()> {
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn write_config(value: serde_json::Value, config_name: &'static str) -> Result<()> {
    async_std::task::spawn_blocking(move || {
        check_create_config_dir()?;

        let mut file = std::fs::File::create(get_config_file(config_name))?;
        file.write_all(value.to_string().as_bytes())?;

        Ok(())
    })
    .await
}
