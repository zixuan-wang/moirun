use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Wry,
};

use crate::settings::AppSettings;
use crate::timer::TimerState;
use crate::window::get_or_create_settings_window;
use std::sync::{Arc, Mutex};

pub fn build_tray_menu(app: &AppHandle) -> Result<Menu<Wry>, String> {
    let menu = Menu::new(app).map_err(|e| e.to_string())?;

    let water_toggle = MenuItem::with_id(app, "water_toggle", "喝水提醒", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let eye_toggle = MenuItem::with_id(app, "eye_toggle", "护眼提醒", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let dnd_off = MenuItem::with_id(app, "dnd_off", "关闭勿扰", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let dnd_30 = MenuItem::with_id(app, "dnd_30", "30 分钟", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let dnd_60 = MenuItem::with_id(app, "dnd_60", "1 小时", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let dnd_submenu = Submenu::with_id_and_items(
        app,
        "dnd",
        "勿扰模式",
        true,
        &[&dnd_off, &dnd_30, &dnd_60],
    )
    .map_err(|e| e.to_string())?;

    let stats_item = MenuItem::with_id(app, "stats", "今日统计", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let settings_item = MenuItem::with_id(app, "settings", "打开设置", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let separator = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    menu.append(&water_toggle).map_err(|e| e.to_string())?;
    menu.append(&eye_toggle).map_err(|e| e.to_string())?;
    menu.append(&dnd_submenu).map_err(|e| e.to_string())?;
    menu.append(&stats_item).map_err(|e| e.to_string())?;
    menu.append(&settings_item).map_err(|e| e.to_string())?;
    menu.append(&separator).map_err(|e| e.to_string())?;
    menu.append(&quit_item).map_err(|e| e.to_string())?;

    Ok(menu)
}

pub fn setup_tray(app: &AppHandle, timer_state: Arc<Mutex<TimerState>>) -> Result<(), String> {
    let menu = build_tray_menu(app)?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "water_toggle" => {
                let store = app.state::<tauri_plugin_store::Store<Wry>>();
                let mut settings = AppSettings::load(&store);
                settings.water_reminder_enabled = !settings.water_reminder_enabled;
                let _ = settings.save(&store);
                if let Ok(mut ts) = timer_state.lock() {
                    ts.water_enabled = settings.water_reminder_enabled;
                }
            }
            "eye_toggle" => {
                let store = app.state::<tauri_plugin_store::Store<Wry>>();
                let mut settings = AppSettings::load(&store);
                settings.eye_care_enabled = !settings.eye_care_enabled;
                let _ = settings.save(&store);
                if let Ok(mut ts) = timer_state.lock() {
                    ts.eye_enabled = settings.eye_care_enabled;
                }
            }
            "dnd_off" => {
                if let Ok(mut ts) = timer_state.lock() {
                    ts.set_dnd(None);
                }
            }
            "dnd_30" => {
                if let Ok(mut ts) = timer_state.lock() {
                    ts.set_dnd(Some(30));
                }
            }
            "dnd_60" => {
                if let Ok(mut ts) = timer_state.lock() {
                    ts.set_dnd(Some(60));
                }
            }
            "stats" => {
                let _ = app.emit("show-stats", ());
            }
            "settings" => {
                let _ = get_or_create_settings_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let _ = get_or_create_settings_window(app);
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}
