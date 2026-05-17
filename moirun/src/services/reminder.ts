import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AppSettings, DailyStats, TimerStatus } from "../types";

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function setSettings(settings: AppSettings): Promise<void> {
  return invoke("set_settings", { settings });
}

export async function getTodayStats(): Promise<DailyStats> {
  return invoke<DailyStats>("get_today_stats");
}

export async function confirmWater(): Promise<DailyStats> {
  return invoke<DailyStats>("confirm_water");
}

export async function confirmEyeCare(): Promise<DailyStats> {
  return invoke<DailyStats>("confirm_eye_care");
}

export async function snoozeEyeCare(minutes: number): Promise<void> {
  return invoke("snooze_eye_care", { minutes });
}

export async function toggleDoNotDisturb(minutes?: number): Promise<void> {
  return invoke("toggle_do_not_disturb", { minutes });
}

export async function openSettingsWindow(): Promise<void> {
  return invoke("open_settings_window");
}

export async function openEyeCareWindow(intensity: string): Promise<void> {
  return invoke("open_eye_care_window", { intensity });
}

export async function closeEyeCare(): Promise<void> {
  return invoke("close_eye_care");
}

export async function getDndStatus(): Promise<boolean> {
  return invoke<boolean>("get_dnd_status");
}

export async function getTimerStatus(): Promise<TimerStatus> {
  return invoke<TimerStatus>("get_timer_status");
}

export function closeCurrentWindow(): Promise<void> {
  return getCurrentWindow().close();
}

export async function onWaterReminder(callback: () => void): Promise<UnlistenFn> {
  const unlisten = await listen("water-reminder", callback);
  return unlisten;
}

export async function onEyeCareReminder(
  callback: (intensity: string) => void
): Promise<UnlistenFn> {
  const unlisten = await listen<string>("eye-care-reminder", (e) => {
    callback(e.payload);
  });
  return unlisten;
}

export async function onStatsUpdated(
  callback: (stats: DailyStats) => void
): Promise<UnlistenFn> {
  const unlisten = await listen<DailyStats>("stats-updated", (e) => {
    callback(e.payload);
  });
  return unlisten;
}

export async function onShowStats(callback: () => void): Promise<UnlistenFn> {
  const unlisten = await listen("show-stats", callback);
  return unlisten;
}

export async function onDoNotDisturbChanged(
  callback: (active: boolean) => void
): Promise<UnlistenFn> {
  const unlisten = await listen<boolean>("do-not-disturb-changed", (e) => {
    callback(e.payload);
  });
  return unlisten;
}
