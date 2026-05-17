use chrono::Local;
use serde::{Deserialize, Serialize};

pub trait KeyValueStore {
    fn get(&self, key: &str) -> Option<serde_json::Value>;
    fn set(&self, key: &str, value: serde_json::Value);
    fn save(&self) -> Result<(), String>;
}

impl KeyValueStore for tauri_plugin_store::Store<tauri::Wry> {
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.get(key)
    }

    fn set(&self, key: &str, value: serde_json::Value) {
        self.set(key, value);
    }

    fn save(&self) -> Result<(), String> {
        tauri_plugin_store::Store::save(self).map_err(|e| e.to_string())
    }
}

impl<T: KeyValueStore> KeyValueStore for std::sync::Arc<T> {
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.as_ref().get(key)
    }

    fn set(&self, key: &str, value: serde_json::Value) {
        self.as_ref().set(key, value)
    }

    fn save(&self) -> Result<(), String> {
        self.as_ref().save()
    }
}

impl<T: KeyValueStore + Send + Sync> KeyValueStore for tauri::State<'_, T> {
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        (**self).get(key)
    }

    fn set(&self, key: &str, value: serde_json::Value) {
        (**self).set(key, value)
    }

    fn save(&self) -> Result<(), String> {
        (**self).save()
    }
}

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
    pub fn load<S: KeyValueStore>(store: &S) -> Self {
        store
            .get("settings")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default()
    }

    pub fn save<S: KeyValueStore>(&self, store: &S) -> Result<(), String> {
        let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
        store.set("settings", value);
        store.save()
    }
}

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
    pub fn load<S: KeyValueStore>(store: &S) -> Self {
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

    pub fn save<S: KeyValueStore>(&self, store: &S) -> Result<(), String> {
        let value = serde_json::to_value(self).map_err(|e| e.to_string())?;
        store.set("stats", value);
        store.save()
    }

    pub fn increment_water(&mut self) {
        self.water_count += 1;
    }

    pub fn increment_eye_care(&mut self) {
        self.eye_care_count += 1;
    }

    pub fn check_and_reset(&mut self, today: &str) -> bool {
        if self.date != today {
            self.date = today.to_string();
            self.water_count = 0;
            self.eye_care_count = 0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
pub struct MemoryStore {
    data: std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>,
}

#[cfg(test)]
impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
}

#[cfg(test)]
impl KeyValueStore for MemoryStore {
    fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.lock().unwrap().get(key).cloned()
    }

    fn set(&self, key: &str, value: serde_json::Value) {
        self.data.lock().unwrap().insert(key.to_string(), value);
    }

    fn save(&self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_settings_round_trip() {
        let store = MemoryStore::new();
        let settings = AppSettings {
            water_reminder_enabled: false,
            water_interval_minutes: 60,
            eye_care_enabled: false,
            eye_care_interval_minutes: 90,
            eye_care_intensity: "strict".to_string(),
            eye_care_lock_seconds: 30,
            auto_start: true,
        };

        settings.save(&store).unwrap();
        let loaded = AppSettings::load(&store);

        assert!(!loaded.water_reminder_enabled);
        assert_eq!(loaded.water_interval_minutes, 60);
        assert_eq!(loaded.eye_care_intensity, "strict");
    }

    #[test]
    fn daily_stats_round_trip() {
        let store = MemoryStore::new();
        let mut stats = DailyStats::default();
        stats.increment_water();
        stats.increment_eye_care();

        stats.save(&store).unwrap();
        let loaded = DailyStats::load(&store);

        assert_eq!(loaded.water_count, 1);
        assert_eq!(loaded.eye_care_count, 1);
    }

    #[test]
    fn daily_stats_default_on_empty_store() {
        let store = MemoryStore::new();
        let stats = DailyStats::load(&store);

        assert_eq!(stats.water_count, 0);
        assert_eq!(stats.eye_care_count, 0);
    }
}
