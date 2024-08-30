use anyhow::{Context, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub enum FinanceManager {
    RAM,
    /// SQLite database and the path to the database file
    SQLite(String),
    /// REST API and the URL to the API
    API(String),
}

#[derive(Debug, Clone)]
pub struct Settings {
    finance_manager: FinanceManager,
}

impl Settings {
    pub fn new(finance_manager: FinanceManager) -> Self {
        Self { finance_manager }
    }

    pub fn finance_manager(&self) -> &FinanceManager {
        &self.finance_manager
    }
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
                    finance_manager: FinanceManager::RAM,
                });
            } else {
                return Err(err.into());
            }
        }
    };

    file.read_to_string(&mut content)?;

    let json_value: serde_json::Value = serde_json::from_str(&content)?;

    let map = json_value
        .as_object()
        .context("Settings file is not an object")?;

    let fm_type = map
        .get("finance_manager")
        .context("Missing 'finance_manager' key")?
        .as_str()
        .context("'finance_manager' is not a string")?;

    // Read settings from file
    Ok(Settings {
        finance_manager: match fm_type {
            "RAM" => FinanceManager::RAM,
            "SQLite" => {
                let path = map
                    .get("finance_manager_info")
                    .context("Missing 'finance_manager_info' key")?
                    .get("path")
                    .context("Missing 'path' key")?
                    .as_str()
                    .context("'path' is not a string")?;
                FinanceManager::SQLite(path.to_string())
            }
            "API" => {
                let url = map
                    .get("finance_manager_info")
                    .context("Missing 'finance_manager_info' key")?
                    .get("url")
                    .context("Missing 'url' key")?
                    .as_str()
                    .context("'url' is not a string")?;
                FinanceManager::API(url.to_string())
            }
            _ => anyhow::bail!("Unknown finance manager type: {}", fm_type),
        },
    })
}

#[cfg(not(feature = "native"))]
pub fn read_settings() -> Result<Settings> {
    Ok(Settings {
        finance_manager: FinanceManager::RAM,
    })
}

#[cfg(feature = "native")]
pub fn write_settings(settings: &Settings) -> Result<()> {
    let mut value = serde_json::Map::new();
    value.insert(
        "finance_manager".to_string(),
        match &settings.finance_manager {
            FinanceManager::RAM => "RAM",
            FinanceManager::SQLite(_) => "SQLite",
            FinanceManager::API(_) => "API",
        }
        .into(),
    );

    let mut fm_info = serde_json::Map::new();
    match &settings.finance_manager {
        FinanceManager::SQLite(path) => {
            fm_info.insert("path".to_string(), path.to_string().into());
        }
        FinanceManager::API(url) => {
            fm_info.insert("url".to_string(), url.to_string().into());
        }
        _ => {}
    }
    value.insert("finance_manager_info".to_string(), fm_info.into());

    let mut file = std::fs::File::create(get_settings_path())?;
    file.write_all(serde_json::Value::Object(value).to_string().as_bytes())?;

    Ok(())
}

#[cfg(not(feature = "native"))]
pub fn write_settings(settings: &Settings) -> Result<()> {
    Ok(())
}