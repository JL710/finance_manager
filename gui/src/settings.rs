use anyhow::Result;

const CONFIG_NAME: &str = "settings";

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

pub async fn read_settings() -> Result<Settings> {
    if let Some(conf) = crate::config::read_config(CONFIG_NAME).await? {
        Ok(serde_json::from_value(conf)?)
    } else {
        Ok(Settings::default())
    }
}

pub async fn write_settings(settings: Settings) -> Result<()> {
    crate::config::write_config(serde_json::to_value(settings)?, CONFIG_NAME).await
}
