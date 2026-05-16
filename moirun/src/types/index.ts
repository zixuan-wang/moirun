export interface AppSettings {
  water_reminder_enabled: boolean;
  water_interval_minutes: number;
  eye_care_enabled: boolean;
  eye_care_interval_minutes: number;
  eye_care_intensity: 'gentle' | 'locked' | 'strict';
  eye_care_lock_seconds: number;
  auto_start: boolean;
}

export interface DailyStats {
  date: string;
  water_count: number;
  eye_care_count: number;
}
