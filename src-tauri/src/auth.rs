use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::helper;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthState {
    pub banned: bool,
    pub id: String,
    pub access_token: Option<String>,
    pub avatar: String,
    pub ingame_name: String,
    pub locale: String,
    pub platform: String,
    pub region: String,
    pub role: String,
}
// Allow us to run AuthState::default()
impl Default for AuthState {
    fn default() -> Self {
        Self {
            banned: false,
            id: "".to_string(),
            access_token: None,
            avatar: "".to_string(),
            ingame_name: "".to_string(),
            locale: "".to_string(),
            platform: "".to_string(),
            region: "".to_string(),
            role: "".to_string(),
        }
    }
}
impl AuthState {
    fn get_file_path() -> PathBuf {
        let app_path = helper::get_app_roaming_path();
        let auth_path = app_path.join("auth.json");
        auth_path
    }
    pub fn setup() -> io::Result<Self> {
        let path_ref = Self::get_file_path();
        if path_ref.exists() {
            Self::read_from_file()
        } else {
            let default_auth = AuthState::default();
            default_auth.save_to_file()?;
            Ok(default_auth)
        }
    }

    pub fn save_to_file(&self) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(Self::get_file_path())?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn read_from_file() -> io::Result<Self> {
        let mut file = File::open(Self::get_file_path())?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let auth = serde_json::from_str(&content)?;
        Ok(auth)
    }
}
