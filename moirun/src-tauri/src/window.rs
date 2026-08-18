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
        tauri::WebviewUrl::App("index.html#/settings".into()),
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
        tauri::WebviewUrl::App("index.html#/eyecare".into()),
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
        // monitor.size()/position() 返回物理像素，需换算为逻辑像素，
        // 否则高 DPI（如 Retina）屏幕上遮罩会成倍放大并错位
        let scale = monitor.scale_factor();
        let size = monitor.size();
        let position = monitor.position();

        let window = tauri::WebviewWindowBuilder::new(
            app,
            &label,
            tauri::WebviewUrl::App("index.html#/overlay".into()),
        )
        .title("")
        .inner_size(size.width as f64 / scale, size.height as f64 / scale)
        .position(position.x as f64 / scale, position.y as f64 / scale)
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
    // 不依赖当前显示器数量推导 label，避免两次提醒之间拔掉显示器导致残留
    for (label, window) in app.webview_windows() {
        if label.starts_with("overlay-") {
            let _ = window.close();
        }
    }
}
