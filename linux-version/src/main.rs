use std::sync::Arc;

use gio::prelude::*;
use herdrbell::config::Settings;
use herdrbell::i18n::{AppLanguage, LocalizationManager};
use herdrbell::core::store::HerdrStore;
use herdrbell::ui::settings::SettingsWindow;
use herdrbell::ui::tray;

fn main() {
    env_logger::init();

    let settings = Arc::new(Settings::new());
    let lang_code = settings.language();
    let lang = AppLanguage::from_code(&lang_code).unwrap_or(AppLanguage::ENGLISH);
    let l10n = Arc::new(LocalizationManager::new(lang));

    let store = Arc::new(HerdrStore::new(settings.clone(), l10n.clone()));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime");

    rt.block_on(async {
        store.start().await;
    });

    let app = gtk4::Application::builder()
        .application_id("dev.herdrbell.HerdrBell")
        .build();

    let settings_for_win = settings.clone();
    let l10n_for_win = l10n.clone();

    app.connect_activate(move |app| {
        SettingsWindow::show(app, settings_for_win.clone(), l10n_for_win.clone());
    });

    let (configure_tx, configure_rx) = std::sync::mpsc::channel::<()>();

    let settings_for_configure = settings.clone();
    let l10n_for_configure = l10n.clone();
    let app_weak = app.downgrade();
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        while configure_rx.try_recv().is_ok() {
            if let Some(app) = app_weak.upgrade() {
                SettingsWindow::show(&app, settings_for_configure.clone(), l10n_for_configure.clone());
            }
        }
        glib::ControlFlow::Continue
    });

    let on_configure = Arc::new(move || {
        let _ = configure_tx.send(());
    });

    let on_quit = Arc::new(move || {
        std::process::exit(0);
    });

    let rt_handle = rt.handle().clone();
    std::thread::spawn(move || {
        rt_handle.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        });
    });

    let _rt_guard = rt.enter();
    let _handle = tray::start_tray(store.clone(), l10n.clone(), on_configure, on_quit);

    app.run();
}
