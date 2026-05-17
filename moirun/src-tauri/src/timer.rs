use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

use crate::persistence::AppSettings;

pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
    fn sleep(&self, duration: Duration);
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

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

    clock: Arc<dyn Clock>,
}

impl TimerState {
    pub fn from_settings(settings: &AppSettings, clock: Arc<dyn Clock>) -> Self {
        let now = clock.now();
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

            clock,
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
        self.water_next = self.clock.now() + self.water_interval;
    }

    pub fn reset_eye_timer(&mut self) {
        self.eye_next = self.clock.now() + self.eye_interval;
        self.eye_snooze_until = None;
    }

    pub fn snooze_eye(&mut self, minutes: u64) {
        self.eye_snooze_until = Some(self.clock.now() + Duration::from_secs(minutes * 60));
    }

    pub fn is_dnd_active(&self) -> bool {
        match self.dnd {
            DoNotDisturbState::Off => false,
            DoNotDisturbState::Until(t) => self.clock.now() < t,
        }
    }

    pub fn set_dnd(&mut self, minutes: Option<u64>) {
        self.dnd = match minutes {
            Some(m) => DoNotDisturbState::Until(self.clock.now() + Duration::from_secs(m * 60)),
            None => DoNotDisturbState::Off,
        };
    }

    pub fn enter_system_pause(&mut self) {
        if self.system_pause_depth == 0 {
            self.system_pause_start = Some(self.clock.now());
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
                let elapsed = self.clock.now().saturating_duration_since(start);
                self.water_next += elapsed;
                self.eye_next += elapsed;
            }
        }
    }

    pub fn is_system_paused(&self) -> bool {
        self.system_pause_depth > 0
    }

    pub fn get_status(&self) -> TimerStatus {
        let now = self.clock.now();
        TimerStatus {
            water_remaining_secs: self.water_next.saturating_duration_since(now).as_secs(),
            eye_remaining_secs: self.eye_next.saturating_duration_since(now).as_secs(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TimerStatus {
    pub water_remaining_secs: u64,
    pub eye_remaining_secs: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReminderEvent {
    Water,
    EyeCare { intensity: String },
}

/// 纯函数：检查一次定时器状态，如有提醒到期则返回事件并更新状态
pub fn tick(state: &mut TimerState) -> Option<ReminderEvent> {
    if state.is_dnd_active() {
        return None;
    }

    if state.is_system_paused() {
        return None;
    }

    let now = state.clock.now();

    if state.water_enabled && now >= state.water_next {
        state.water_next = now + state.water_interval;
        return Some(ReminderEvent::Water);
    }

    let snooze_expired = state.eye_snooze_until.map_or(true, |t| now >= t);

    if state.eye_enabled && snooze_expired && now >= state.eye_next {
        let intensity = state.eye_intensity.clone();
        state.eye_next = now + state.eye_interval;
        state.eye_snooze_until = None;
        return Some(ReminderEvent::EyeCare { intensity });
    }

    None
}

pub fn spawn_timer_loop(
    app_handle: AppHandle,
    state: Arc<Mutex<TimerState>>,
    clock: Arc<dyn Clock>,
) {
    std::thread::spawn(move || loop {
        clock.sleep(Duration::from_secs(1));

        let mut st = state.lock().unwrap();
        let event = tick(&mut st);
        drop(st);

        match event {
            Some(ReminderEvent::Water) => {
                let _ = app_handle.emit("water-reminder", ());
            }
            Some(ReminderEvent::EyeCare { intensity }) => {
                let _ = app_handle.emit("eye-care-reminder", intensity);
            }
            None => {}
        }
    });
}

#[cfg(test)]
pub struct MockClock {
    now: Mutex<Instant>,
}

#[cfg(test)]
impl MockClock {
    pub fn new(start: Instant) -> Self {
        Self {
            now: Mutex::new(start),
        }
    }

    pub fn advance(&self, duration: Duration) {
        *self.now.lock().unwrap() += duration;
    }
}

#[cfg(test)]
impl Clock for MockClock {
    fn now(&self) -> Instant {
        *self.now.lock().unwrap()
    }

    fn sleep(&self, _duration: Duration) {
        // no-op in tests
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_state_from_settings() {
        let clock = Arc::new(MockClock::new(Instant::now()));
        let settings = AppSettings {
            water_reminder_enabled: true,
            water_interval_minutes: 30,
            eye_care_enabled: true,
            eye_care_interval_minutes: 45,
            eye_care_intensity: "gentle".to_string(),
            eye_care_lock_seconds: 20,
            auto_start: false,
        };
        let ts = TimerState::from_settings(&settings, clock.clone());
        assert!(ts.water_enabled);
        assert!(ts.eye_enabled);
        assert_eq!(ts.water_interval, Duration::from_secs(30 * 60));
        assert_eq!(ts.eye_interval, Duration::from_secs(45 * 60));
    }

    #[test]
    fn dnd_state() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        let settings = AppSettings::default();
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        assert!(!ts.is_dnd_active());

        ts.set_dnd(Some(30));
        assert!(ts.is_dnd_active());

        clock.advance(Duration::from_secs(31 * 60));
        assert!(!ts.is_dnd_active());
    }

    #[test]
    fn system_pause_adds_back_elapsed() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        let settings = AppSettings::default();
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        let water_next_before = ts.water_next;
        ts.enter_system_pause();
        clock.advance(Duration::from_secs(60));
        ts.exit_system_pause();

        assert_eq!(ts.water_next, water_next_before + Duration::from_secs(60));
    }

    #[test]
    fn snooze_eye_sets_snooze_from_now() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        let settings = AppSettings::default();
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        let now_before = clock.now();
        ts.snooze_eye(5);

        assert!(ts.eye_snooze_until.is_some());
        assert_eq!(ts.eye_snooze_until.unwrap(), now_before + Duration::from_secs(5 * 60));
    }

    #[test]
    fn tick_triggers_water_when_due() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        let settings = AppSettings::default();
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        // 未到时间，不触发
        assert_eq!(tick(&mut ts), None);

        // 推进时间越过 water_next
        clock.advance(ts.water_interval + Duration::from_secs(1));
        let water_next_before = ts.water_next;
        assert_eq!(tick(&mut ts), Some(ReminderEvent::Water));
        // 触发后 water_next 应已重置
        assert!(ts.water_next > water_next_before);
    }

    #[test]
    fn tick_triggers_eye_care_when_due() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        // water 间隔设长，避免与 eye 干扰
        let settings = AppSettings {
            water_interval_minutes: 60,
            eye_care_intensity: "strict".to_string(),
            ..AppSettings::default()
        };
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        assert_eq!(tick(&mut ts), None);

        clock.advance(ts.eye_interval + Duration::from_secs(1));
        let eye_next_before = ts.eye_next;
        assert_eq!(
            tick(&mut ts),
            Some(ReminderEvent::EyeCare {
                intensity: "strict".to_string()
            })
        );
        assert!(ts.eye_next > eye_next_before);
        assert!(ts.eye_snooze_until.is_none());
    }

    #[test]
    fn tick_dnd_blocks_all_events() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        let settings = AppSettings::default();
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        ts.set_dnd(Some(60)); // DND 设长，确保持续有效
        clock.advance(ts.water_interval + Duration::from_secs(1));

        assert_eq!(tick(&mut ts), None);
    }

    #[test]
    fn tick_system_pause_blocks_all_events() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        let settings = AppSettings::default();
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        ts.enter_system_pause();
        clock.advance(ts.water_interval + Duration::from_secs(1));

        assert_eq!(tick(&mut ts), None);
    }

    #[test]
    fn tick_eye_snooze_delays_reminder() {
        let start = Instant::now();
        let clock = Arc::new(MockClock::new(start));
        let settings = AppSettings {
            water_interval_minutes: 60, // 避免 water 干扰
            ..AppSettings::default()
        };
        let mut ts = TimerState::from_settings(&settings, clock.clone());

        // 推进到 eye_next 前 10 秒
        clock.advance(ts.eye_interval - Duration::from_secs(10));
        // 设置 snooze 5 分钟
        ts.snooze_eye(5);

        // 再推进 10 秒，越过 eye_next，但 snooze 仍在有效期内
        clock.advance(Duration::from_secs(10));
        assert_eq!(tick(&mut ts), None);

        // 越过 snooze 结束时间（再推进 5 分钟）
        clock.advance(Duration::from_secs(5 * 60));
        assert_eq!(tick(&mut ts), Some(ReminderEvent::EyeCare { intensity: "gentle".to_string() }));
    }
}
