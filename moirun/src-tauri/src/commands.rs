// 命令内核 —— 纯业务逻辑，不依赖 Tauri 框架类型
// 可独立测试，仅以领域对象（DailyStats, TimerState）为参数

use crate::persistence::{AppSettings, DailyStats};
use crate::timer::TimerState;

/// 确认饮水：增加计数并重置计时器
pub fn confirm_water_core(stats: &mut DailyStats, timer_state: &mut TimerState) {
    stats.increment_water();
    timer_state.reset_water_timer();
}

/// 确认护眼：增加计数
pub fn confirm_eye_care_core(stats: &mut DailyStats) {
    stats.increment_eye_care();
}

/// 延后护眼
pub fn snooze_eye_care_core(timer_state: &mut TimerState, minutes: u64) {
    timer_state.snooze_eye(minutes);
}

/// 切换勿扰模式
pub fn toggle_dnd_core(timer_state: &mut TimerState, minutes: Option<u64>) {
    timer_state.set_dnd(minutes);
}

/// 应用设置并同步至计时器状态
pub fn apply_settings_core(settings: &AppSettings, timer_state: &mut TimerState) {
    timer_state.update_from_settings(settings);
}

/// 切换喝水提醒开关（纯函数，不涉 I/O）
pub fn toggle_water_core(settings: &mut AppSettings, timer_state: &mut TimerState) {
    settings.water_reminder_enabled = !settings.water_reminder_enabled;
    apply_settings_core(settings, timer_state);
}

/// 切换护眼提醒开关（纯函数，不涉 I/O）
pub fn toggle_eye_core(settings: &mut AppSettings, timer_state: &mut TimerState) {
    settings.eye_care_enabled = !settings.eye_care_enabled;
    apply_settings_core(settings, timer_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::AppSettings;
    use crate::timer::{MockClock, TimerState};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn test_settings() -> AppSettings {
        AppSettings::default()
    }

    fn test_state() -> (TimerState, Arc<MockClock>) {
        let clock = Arc::new(MockClock::new(Instant::now()));
        let ts = TimerState::from_settings(&test_settings(), clock.clone());
        (ts, clock)
    }

    #[test]
    fn confirm_water_increments_counter() {
        let mut stats = DailyStats::default();
        let (mut ts, clock) = test_state();
        let water_next_before = ts.water_next;

        // 推进时间，使 reset 后的 water_next 与之前不同
        clock.advance(Duration::from_secs(1));
        confirm_water_core(&mut stats, &mut ts);

        assert_eq!(stats.water_count, 1);
        assert_ne!(ts.water_next, water_next_before);
    }

    #[test]
    fn confirm_eye_care_increments_counter() {
        let mut stats = DailyStats::default();
        confirm_eye_care_core(&mut stats);
        assert_eq!(stats.eye_care_count, 1);
    }

    #[test]
    fn snooze_eye_sets_snooze() {
        let (mut ts, _clock) = test_state();
        snooze_eye_care_core(&mut ts, 5);
        assert!(ts.eye_snooze_until.is_some());
    }

    #[test]
    fn toggle_dnd_turns_on_and_off() {
        let (mut ts, _clock) = test_state();
        assert!(!ts.is_dnd_active());

        toggle_dnd_core(&mut ts, Some(30));
        assert!(ts.is_dnd_active());

        toggle_dnd_core(&mut ts, None);
        assert!(!ts.is_dnd_active());
    }

    #[test]
    fn apply_settings_syncs_to_timer() {
        let (mut ts, _clock) = test_state();
        let new_settings = AppSettings {
            water_reminder_enabled: false,
            water_interval_minutes: 60,
            eye_care_enabled: false,
            eye_care_interval_minutes: 90,
            eye_care_intensity: "strict".to_string(),
            eye_care_lock_seconds: 30,
            auto_start: true,
        };

        apply_settings_core(&new_settings, &mut ts);

        assert!(!ts.water_enabled);
        assert!(!ts.eye_enabled);
        assert_eq!(ts.eye_intensity, "strict");
    }
}
