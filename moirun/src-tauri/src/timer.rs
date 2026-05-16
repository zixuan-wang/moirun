use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use crate::settings::AppSettings;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DoNotDisturbState {
    Off,
    Until(Instant),
}

pub struct TimerState {
    pub water_enabled: bool,
    pub water_interval: Duration,
    pub water_next: Instant,

    pub eye_enabled: bool,
    pub eye_interval: Duration,
    pub eye_next: Instant,
    pub eye_intensity: String,

    pub dnd: DoNotDisturbState,
    pub eye_snooze_until: Option<Instant>,

    pub system_pause_start: Option<Instant>,
    pub system_pause_depth: u32,
}

impl TimerState {
    pub fn from_settings(settings: &AppSettings) -> Self {
        let now = Instant::now();
        Self {
            water_enabled: settings.water_reminder_enabled,
            water_interval: Duration::from_secs(settings.water_interval_minutes * 60),
            water_next: now + Duration::from_secs(settings.water_interval_minutes * 60),

            eye_enabled: settings.eye_care_enabled,
            eye_interval: Duration::from_secs(settings.eye_care_interval_minutes * 60),
            eye_next: now + Duration::from_secs(settings.eye_care_interval_minutes * 60),
            eye_intensity: settings.eye_care_intensity.clone(),

            dnd: DoNotDisturbState::Off,
            eye_snooze_until: None,

            system_pause_start: None,
            system_pause_depth: 0,
        }
    }

    pub fn update_from_settings(&mut self, settings: &AppSettings) {
        self.water_enabled = settings.water_reminder_enabled;
        self.water_interval = Duration::from_secs(settings.water_interval_minutes * 60);
        self.eye_enabled = settings.eye_care_enabled;
        self.eye_interval = Duration::from_secs(settings.eye_care_interval_minutes * 60);
        self.eye_intensity = settings.eye_care_intensity.clone();
    }

    pub fn reset_water_timer(&mut self) {
        self.water_next = Instant::now() + self.water_interval;
    }

    pub fn reset_eye_timer(&mut self) {
        self.eye_next = Instant::now() + self.eye_interval;
        self.eye_snooze_until = None;
    }

    pub fn snooze_eye(&mut self, minutes: u64) {
        self.eye_snooze_until = Some(Instant::now() + Duration::from_secs(minutes * 60));
    }

    pub fn is_dnd_active(&self) -> bool {
        match self.dnd {
            DoNotDisturbState::Off => false,
            DoNotDisturbState::Until(t) => Instant::now() < t,
        }
    }

    pub fn set_dnd(&mut self, minutes: Option<u64>) {
        self.dnd = match minutes {
            Some(m) => DoNotDisturbState::Until(Instant::now() + Duration::from_secs(m * 60)),
            None => DoNotDisturbState::Off,
        };
    }

    pub fn enter_system_pause(&mut self) {
        if self.system_pause_depth == 0 {
            self.system_pause_start = Some(Instant::now());
        }
        self.system_pause_depth += 1;
    }

    pub fn exit_system_pause(&mut self) {
        if self.system_pause_depth == 0 {
            return;
        }
        self.system_pause_depth -= 1;
        if self.system_pause_depth == 0 {
            if let Some(start) = self.system_pause_start.take() {
                let elapsed = Instant::now().saturating_duration_since(start);
                self.water_next += elapsed;
                self.eye_next += elapsed;
            }
        }
    }

    pub fn is_system_paused(&self) -> bool {
        self.system_pause_depth > 0
    }
}

pub fn spawn_timer_loop(app_handle: AppHandle, state: Arc<Mutex<TimerState>>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));

        let mut st = state.lock().unwrap();

        if st.is_dnd_active() {
            continue;
        }

        if st.is_system_paused() {
            continue;
        }

        let now = Instant::now();

        if st.water_enabled && now >= st.water_next {
            st.water_next = now + st.water_interval;
            drop(st);
            let _ = app_handle.emit("water-reminder", ());
            continue;
        }

        let snooze_expired = st
            .eye_snooze_until
            .map_or(true, |t| now >= t);

        if st.eye_enabled && snooze_expired && now >= st.eye_next {
            let intensity = st.eye_intensity.clone();
            st.eye_next = now + st.eye_interval;
            st.eye_snooze_until = None;
            drop(st);
            let _ = app_handle.emit("eye-care-reminder", intensity);
            continue;
        }
    });
}
