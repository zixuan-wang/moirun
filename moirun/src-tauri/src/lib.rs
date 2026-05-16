use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;

mod power_monitor;
mod settings;
mod stats;
mod timer;
mod tray;
mod window;

use settings::AppSettings;
use stats::DailyStats;
use timer::{spawn_timer_loop, TimerState};
use window::{close_all_overlay_windows, close_eyecare_window, get_or_create_eyecare_window, get_or_create_settings_window};

#[tauri::command]
fn get_settings(store: tauri::State<'_, tauri_plugin_store::Store<tauri::Wry>>) -> Result<AppSettings, String> {
    Ok(AppSettings::load(&store))
}

#[tauri::command]
fn set_settings(
    settings: AppSettings,
    store: tauri::State<'_, tauri_plugin_store::Store<tauri::Wry>>,
    timer_state: tauri::State<'_, Arc<Mutex<TimerState>>>,
) -> Result<(), String> {
    settings.save(&store)?;
    if let Ok(mut ts) = timer_state.lock() {
        ts.update_from_settings(&settings);
    }
    Ok(())
}

#[tauri::command]
fn get_today_stats(store: tauri::State<'_, tauri_plugin_store::Store<tauri::Wry>>) -> Result<DailyStats, String> {
    Ok(DailyStats::load(&store))
}

#[tauri::command]
fn confirm_water(
    store: tauri::State<'_, tauri_plugin_store::Store<tauri::Wry>>,
    timer_state: tauri::State<'_, Arc<Mutex<TimerState>>>,
    app: tauri::AppHandle,
) -> Result<DailyStats, String> {
    let mut stats = DailyStats::load(&store);
    stats.increment_water();
    stats.save(&store)?;

    if let Ok(mut ts) = timer_state.lock() {
        ts.reset_water_timer();
    }

    let _ = app.emit("stats-updated", &stats);
    Ok(stats)
}

#[tauri::command]
fn confirm_eye_care(
    store: tauri::State<'_, tauri_plugin_store::Store<tauri::Wry>>,
    app: tauri::AppHandle,
) -> Result<DailyStats, String> {
    let mut stats = DailyStats::load(&store);
    stats.increment_eye_care();
    stats.save(&store)?;

    close_eyecare_window(&app);
    close_all_overlay_windows(&app);

    let _ = app.emit("stats-updated", &stats);
    Ok(stats)
}

#[tauri::command]
fn snooze_eye_care(
    minutes: u64,
    timer_state: tauri::State<'_, Arc<Mutex<TimerState>>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if let Ok(mut ts) = timer_state.lock() {
        ts.snooze_eye(minutes);
    }
    close_eyecare_window(&app);
    close_all_overlay_windows(&app);
    Ok(())
}

#[tauri::command]
fn toggle_do_not_disturb(
    minutes: Option<u64>,
    timer_state: tauri::State<'_, Arc<Mutex<TimerState>>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    if let Ok(mut ts) = timer_state.lock() {
        ts.set_dnd(minutes);
    }
    let _ = app.emit("do-not-disturb-changed", minutes.is_some());
    Ok(())
}

#[tauri::command]
fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    get_or_create_settings_window(&app)?;
    Ok(())
}

#[tauri::command]
fn open_eye_care_window(app: tauri::AppHandle, intensity: String) -> Result<(), String> {
    if intensity == "strict" {
        let _ = window::create_overlay_windows(&app);
    }
    get_or_create_eyecare_window(&app)?;
    Ok(())
}

#[tauri::command]
fn close_eye_care(app: tauri::AppHandle) -> Result<(), String> {
    close_eyecare_window(&app);
    close_all_overlay_windows(&app);
    Ok(())
}

#[tauri::command]
fn get_dnd_status(timer_state: tauri::State<'_, Arc<Mutex<TimerState>>>) -> Result<bool, String> {
    let ts = timer_state.lock().map_err(|e| e.to_string())?;
    Ok(ts.is_dnd_active())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_today_stats,
            confirm_water,
            confirm_eye_care,
            snooze_eye_care,
            toggle_do_not_disturb,
            open_settings_window,
            open_eye_care_window,
            close_eye_care,
            get_dnd_status,
        ])
        .setup(|app| {
            let store = app.store("moirun.json")?;
            app.manage(store.clone());

            let settings = AppSettings::load(&store);
            let timer_state = Arc::new(Mutex::new(TimerState::from_settings(&settings)));
            app.manage(timer_state.clone());

            tray::setup_tray(app.handle(), timer_state.clone())?;
            spawn_timer_loop(app.handle().clone(), timer_state.clone());

            let ts_suspend = timer_state.clone();
            let ts_resume = timer_state.clone();
            power_monitor::init(
                move || {
                    if let Ok(mut ts) = ts_suspend.lock() {
                        ts.enter_system_pause();
                    }
                },
                move || {
                    if let Ok(mut ts) = ts_resume.lock() {
                        ts.exit_system_pause();
                    }
                },
            );

            let app_handle = app.handle().clone();
            let store_clone = store.clone();
            std::thread::spawn(move || {
                let mut last_date = chrono::Local::now().format("%Y-%m-%d").to_string();
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(300));
                    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                    if today != last_date {
                        last_date = today;
                        let mut stats = DailyStats::load(&store_clone);
                        if stats.check_and_reset() {
                            let _ = stats.save(&store_clone);
                            let _ = app_handle.emit("stats-updated", &stats);
                        }
                    }
                }
            });

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
