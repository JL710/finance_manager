use anyhow::Result;
use std::io::{Read, Write};

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FinanceManager {
    pub sqlite_path: String,
    pub server_url: String,
    pub server_token: String,
    pub selected_finance_manager: SelectedFinanceManager,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SelectedFinanceManager {
    #[default]
    Ram,
    SQLite,
    Server,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub finance_manager: FinanceManager,
    pub utc_seconds_offset: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            finance_manager: FinanceManager::default(),
            utc_seconds_offset: fm_core::get_local_timezone().map_or(0, |x| x.whole_seconds()),
        }
    }
}

fn get_settings_path() -> std::path::PathBuf {
    get_config_path().join("fm_gui_settings.json")
}

fn get_config_path() -> std::path::PathBuf {
    dirs::config_dir().unwrap().join("finance_manager")
}

#[cfg(feature = "native")]
pub fn read_settings() -> Result<Settings> {
    let mut content = String::new();
    let mut file = match std::fs::File::open(get_settings_path()) {
        Ok(file) => file,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Ok(Settings::default());
            } else {
                return Err(err.into());
            }
        }
    };

    file.read_to_string(&mut content)?;

    Ok(serde_json::from_str(&content)?)
}

#[cfg(not(feature = "native"))]
pub fn read_settings() -> Result<Settings> {
    Ok(Settings {
        finance_manager: FinanceManager::default(),
    })
}

#[cfg(feature = "native")]
pub async fn write_settings(settings: Settings) -> Result<()> {
    async_std::task::spawn_blocking(move || {
        if !get_config_path().exists() {
            std::fs::create_dir(get_config_path())?;
        }

        let mut file = std::fs::File::create(get_settings_path())?;
        file.write_all(serde_json::to_value(settings)?.to_string().as_bytes())?;

        Ok(())
    })
    .await
}

#[cfg(not(feature = "native"))]
pub async fn write_settings(settings: Settings) -> Result<()> {
    Ok(())
}
