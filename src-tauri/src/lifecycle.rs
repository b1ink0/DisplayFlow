use crate::store::Shared;
use tauri::{AppHandle, Manager, Runtime};

fn open<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_skip_taskbar(false);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
pub fn hide<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        #[cfg(windows)]
        {
            let _ = window.set_skip_taskbar(true);
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }
}
#[cfg(target_os = "macos")]
fn template_icon() -> tauri::image::Image<'static> {
    let mut data = vec![0u8; 32 * 32 * 4];
    for y in 0..32usize {
        for x in 0..32usize {
            let edge = (5..=26).contains(&x) && (y == 7 || y == 8 || y == 22 || y == 23)
                || (7..=23).contains(&y) && (x == 5 || x == 6 || x == 25 || x == 26);
            let stand =
                (x == 15 || x == 16) && (23..=27).contains(&y) || (11..=20).contains(&x) && y == 27;
            if edge || stand {
                data[(y * 32 + x) * 4 + 3] = 255;
            }
        }
    }
    tauri::image::Image::new_owned(data, 32, 32)
}
pub fn install(app: &mut tauri::App) -> tauri::Result<()> {
    let show = tauri::menu::MenuItem::with_id(app, "show", "Open DisplayFlow", true, None::<&str>)?;
    let pause =
        tauri::menu::MenuItem::with_id(app, "pause", "Pause / resume routing", true, None::<&str>)?;
    let hide_item = tauri::menu::MenuItem::with_id(app, "hide", "Hide window", true, None::<&str>)?;
    let quit = tauri::menu::MenuItem::with_id(app, "quit", "Quit DisplayFlow", true, None::<&str>)?;
    let menu = tauri::menu::Menu::with_items(app, &[&show, &pause, &hide_item, &quit])?;
    let mut tray = tauri::tray::TrayIconBuilder::with_id("displayflow-agent")
        .menu(&menu)
        .tooltip("DisplayFlow")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => open(app),
            "hide" => {
                hide(app);
            }
            "pause" => {
                let state = app.state::<Shared>();
                {
                    let mut config = state.settings.lock().unwrap();
                    config.paused = !config.paused;
                }
                if let Err(e) = state.save() {
                    state.log("error", e);
                }
            }
            "quit" => app.exit(0),
            _ => {}
        });
    #[cfg(target_os = "macos")]
    {
        tray = tray.icon(template_icon()).icon_as_template(true);
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(icon) = app.default_window_icon() {
            tray = tray.icon(icon.clone());
        }
    }
    tray.build(app)?;
    #[cfg(target_os = "macos")]
    {
        use tauri::menu::{Menu, PredefinedMenuItem, Submenu};
        let app_menu = Submenu::with_items(
            app,
            "DisplayFlow",
            true,
            &[
                &PredefinedMenuItem::about(app, Some("About DisplayFlow"), None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::hide(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::quit(app, Some("Quit DisplayFlow"))?,
            ],
        )?;
        let edit = Submenu::with_items(
            app,
            "Edit",
            true,
            &[
                &PredefinedMenuItem::undo(app, None)?,
                &PredefinedMenuItem::redo(app, None)?,
                &PredefinedMenuItem::separator(app)?,
                &PredefinedMenuItem::cut(app, None)?,
                &PredefinedMenuItem::copy(app, None)?,
                &PredefinedMenuItem::paste(app, None)?,
                &PredefinedMenuItem::select_all(app, None)?,
            ],
        )?;
        app.set_menu(Menu::with_items(app, &[&app_menu, &edit])?)?;
    }
    Ok(())
}
pub fn reopen(app: &tauri::AppHandle) {
    open(app)
}
