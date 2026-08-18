use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Wry,
};

use crate::commands;
use crate::persistence::AppSettings;
use crate::timer::TimerState;
use crate::window::get_or_create_settings_window;
use std::sync::{Arc, Mutex};

/// 托盘控制器 —— 封装托盘菜单事件与领域对象之交互
pub struct TrayController {
    timer_state: Arc<Mutex<TimerState>>,
}

impl TrayController {
    pub fn new(timer_state: Arc<Mutex<TimerState>>) -> Self {
        Self { timer_state }
    }

    pub fn toggle_water(&self, app: &AppHandle) {
        let store = app.state::<Arc<tauri_plugin_store::Store<Wry>>>();
        let mut settings = AppSettings::load(&store);
        if let Ok(mut ts) = self.timer_state.lock() {
            commands::toggle_water_core(&mut settings, &mut ts);
        }
        let _ = settings.save(&store);
        // 通知已打开的设置窗口刷新 UI
        let _ = app.emit("settings-changed", &settings);
    }

    pub fn toggle_eye(&self, app: &AppHandle) {
        let store = app.state::<Arc<tauri_plugin_store::Store<Wry>>>();
        let mut settings = AppSettings::load(&store);
        if let Ok(mut ts) = self.timer_state.lock() {
            commands::toggle_eye_core(&mut settings, &mut ts);
        }
        let _ = settings.save(&store);
        let _ = app.emit("settings-changed", &settings);
    }

    pub fn set_dnd(&self, app: &AppHandle, minutes: Option<u64>) {
        if let Ok(mut ts) = self.timer_state.lock() {
            commands::toggle_dnd_core(&mut ts, minutes);
        }
        let _ = app.emit("do-not-disturb-changed", minutes.is_some());
    }

    pub fn show_stats(&self, app: &AppHandle) {
        let _ = app.emit("show-stats", ());
    }

    pub fn open_settings(&self, app: &AppHandle) {
        let _ = get_or_create_settings_window(app);
    }

    pub fn quit(&self, app: &AppHandle) {
        app.exit(0);
    }
}

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
    let controller = TrayController::new(timer_state);

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        // 左键打开设置窗口（见 on_tray_icon_event），右键弹菜单；
        // 不要同时开启 show_menu_on_left_click，否则左键会既弹菜单又开窗
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "water_toggle" => controller.toggle_water(app),
            "eye_toggle" => controller.toggle_eye(app),
            "dnd_off" => controller.set_dnd(app, None),
            "dnd_30" => controller.set_dnd(app, Some(30)),
            "dnd_60" => controller.set_dnd(app, Some(60)),
            "stats" => controller.show_stats(app),
            "settings" => controller.open_settings(app),
            "quit" => controller.quit(app),
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
