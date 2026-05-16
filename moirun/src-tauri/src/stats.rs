use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub water_count: u32,
    pub eye_care_count: u32,
}

impl Default for DailyStats {
    fn default() -> Self {
        Self {
            date: Local::now().format("%Y-%m-%d").to_string(),
            water_count: 0,
            eye_care_count: 0,
        }
    }
}

impl DailyStats {
    pub fn load(store: &tauri_plugin_store::Store<tauri::Wry>) -> Self {
        let today = Local::now().format("%Y-%m-%d").to_string();
        store
            .get("stats")
            .and_then(|v| serde_json::from_value(v).ok())
            .filter(|s: &DailyStats| s.date == today)
            .unwrap_or_else(|| Self {
                date: today,
                ..Default::default()
            })
    }

    pub fn save(&self, store: &tauri_plugin_store::Store<tauri::Wry>) -> Result<(), String> {
        let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
        store.set("stats", value);
        store.save().map_err(|e| e.to_string())
    }

    pub fn increment_water(&mut self) {
        self.water_count += 1;
    }

    pub fn increment_eye_care(&mut self) {
        self.eye_care_count += 1;
    }

    pub fn check_and_reset(&mut self) -> bool {
        let today = Local::now().format("%Y-%m-%d").to_string();
        if self.date != today {
            self.date = today;
            self.water_count = 0;
            self.eye_care_count = 0;
            true
        } else {
            false
        }
    }
}
