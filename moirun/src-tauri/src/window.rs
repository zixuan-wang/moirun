use tauri::{AppHandle, Manager, WebviewWindow};

pub fn get_or_create_settings_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(window);
    }

    let window = tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("/settings".into()),
    )
    .title("设置")
    .inner_size(520.0, 640.0)
    .center()
    .decorations(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(window)
}

pub fn get_or_create_eyecare_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("eyecare") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(window);
    }

    let window = tauri::WebviewWindowBuilder::new(
        app,
        "eyecare",
        tauri::WebviewUrl::App("/eyecare".into()),
    )
    .title("护眼提醒")
    .inner_size(420.0, 320.0)
    .center()
    .decorations(false)
    .always_on_top(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(window)
}

pub fn close_eyecare_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("eyecare") {
        let _ = window.close();
    }
}

pub fn create_overlay_windows(app: &AppHandle) -> Result<Vec<WebviewWindow>, String> {
    let monitors = app.available_monitors().map_err(|e| e.to_string())?;
    let mut overlays = Vec::new();

    for (i, monitor) in monitors.iter().enumerate() {
        let label = format!("overlay-{}", i);
        let size = monitor.size();
        let position = monitor.position();

        let window = tauri::WebviewWindowBuilder::new(
            app,
            &label,
            tauri::WebviewUrl::App("/overlay".into()),
        )
        .title("")
        .inner_size(size.width as f64, size.height as f64)
        .position(position.x as f64, position.y as f64)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .build()
        .map_err(|e| e.to_string())?;

        overlays.push(window);
    }

    Ok(overlays)
}

pub fn close_all_overlay_windows(app: &AppHandle) {
    let monitors = match app.available_monitors() {
        Ok(m) => m,
        Err(_) => return,
    };

    for i in 0..monitors.len() {
        let label = format!("overlay-{}", i);
        if let Some(window) = app.get_webview_window(&label) {
            let _ = window.close();
        }
    }
}
