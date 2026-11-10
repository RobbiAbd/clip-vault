use anyhow::Result;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager, Wry,
};

use crate::settings::Language;

pub struct TrayMenu {
    show: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

impl TrayMenu {
    pub fn set_language(&self, language: Language) {
        let (show, quit) = labels(language);
        let _ = self.show.set_text(show);
        let _ = self.quit.set_text(quit);
    }
}

fn labels(language: Language) -> (&'static str, &'static str) {
    match language.resolve() {
        Language::Id => ("Tampilkan Clip-Vault", "Keluar"),
        _ => ("Show Clip-Vault", "Quit"),
    }
}

pub fn setup(app: &mut App, language: Language) -> Result<()> {
    let (show_label, quit_label) = labels(language);
    let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    TrayIconBuilder::new()
        .menu(&menu)
        .icon(app.default_window_icon().unwrap().clone())
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                crate::window::toggle_main(tray.app_handle(), false);
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => crate::window::show_main(app, false),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    app.manage(TrayMenu { show, quit });
    Ok(())
}
