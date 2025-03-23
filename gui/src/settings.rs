use anyhow::Result;
use std::io::Read;

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

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub finance_manager: FinanceManager,
}

fn get_settings_path() -> std::path::PathBuf {
    dirs::home_dir().unwrap().join(".fm_gui_settings.json")
}

#[cfg(feature = "native")]
pub fn read_settings() -> Result<Settings> {
    let mut content = String::new();
    let mut file = match std::fs::File::open(get_settings_path()) {
        Ok(file) => file,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                return Ok(Settings {
                    finance_manager: FinanceManager::default(),
                });
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
    use futures_lite::io::AsyncWriteExt;

    let mut file = smol::fs::File::create(get_settings_path()).await?;
    file.write_all(serde_json::to_value(settings)?.to_string().as_bytes())
        .await?;
    Ok(())
}

#[cfg(not(feature = "native"))]
pub async fn write_settings(settings: Settings) -> Result<()> {
    Ok(())
}
