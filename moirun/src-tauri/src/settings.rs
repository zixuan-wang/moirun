use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub water_reminder_enabled: bool,
    pub water_interval_minutes: u64,
    pub eye_care_enabled: bool,
    pub eye_care_interval_minutes: u64,
    pub eye_care_intensity: String,
    pub eye_care_lock_seconds: u64,
    pub auto_start: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            water_reminder_enabled: true,
            water_interval_minutes: 30,
            eye_care_enabled: true,
            eye_care_interval_minutes: 30,
            eye_care_intensity: "gentle".to_string(),
            eye_care_lock_seconds: 20,
            auto_start: false,
        }
    }
}

impl AppSettings {
    pub fn load(store: &tauri_plugin_store::Store<tauri::Wry>) -> Self {
        store
            .get("settings")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, store: &tauri_plugin_store::Store<tauri::Wry>) -> Result<(), String> {
        let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
        store.set("settings", value);
        store.save().map_err(|e| e.to_string())
    }
}
