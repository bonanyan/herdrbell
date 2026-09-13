use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, CheckButton, ComboBoxText, Label, Orientation, Separator};

use crate::config::Settings;
use crate::i18n::{AppLanguage, LocalizationManager};
use crate::platform::autostart;
use crate::ui::icons::IconSchemeRegistry;

pub struct SettingsWindow;

impl SettingsWindow {
    pub fn show(app: &Application, settings: Arc<Settings>, l10n: Arc<LocalizationManager>) {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("HerdrBell")
            .default_width(420)
            .default_height(400)
            .resizable(false)
            .build();

        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(16);
        content.set_margin_bottom(16);
        content.set_margin_start(16);
        content.set_margin_end(16);

        let lang_label = Label::new(Some(&l10n.string("settings.language", &[])));
        lang_label.set_halign(gtk4::Align::Start);
        content.append(&lang_label);

        let lang_combo = ComboBoxText::new();
        for lang in AppLanguage::all() {
            lang_combo.append(Some(lang.code()), &lang.native_name());
        }
        lang_combo.set_active_id(Some(&l10n.current_language().code()));
        let l10n_clone = l10n.clone();
        let settings_clone = settings.clone();
        lang_combo.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                let code = id.to_string();
                if let Some(lang) = AppLanguage::from_code(&code) {
                    l10n_clone.set_language(lang);
                    settings_clone.set_language(&code);
                }
            }
        });
        content.append(&lang_combo);

        let lang_help = Label::new(Some(&l10n.string("settings.language.help", &[])));
        lang_help.set_halign(gtk4::Align::Start);
        lang_help.add_css_class("dim-label");
        content.append(&lang_help);

        content.append(&Separator::new(Orientation::Horizontal));

        let scheme_label = Label::new(Some(&l10n.string("settings.iconStyle", &[])));
        scheme_label.set_halign(gtk4::Align::Start);
        content.append(&scheme_label);

        let scheme_combo = ComboBoxText::new();
        for (id, name_key) in IconSchemeRegistry::scheme_ids() {
            let display_name = l10n.string(&name_key, &[]);
            scheme_combo.append(Some(&id), &display_name);
        }
        scheme_combo.set_active_id(Some(&settings.icon_scheme_id()));
        let settings_clone2 = settings.clone();
        scheme_combo.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                settings_clone2.set_icon_scheme_id(&id);
            }
        });
        content.append(&scheme_combo);

        let scheme_help = Label::new(Some(&l10n.string("settings.iconStyle.help", &[])));
        scheme_help.set_halign(gtk4::Align::Start);
        scheme_help.add_css_class("dim-label");
        content.append(&scheme_help);

        content.append(&Separator::new(Orientation::Horizontal));

        let autostart_check = CheckButton::with_label(&l10n.string("settings.launchAtLogin", &[]));
        autostart_check.set_active(autostart::is_enabled());
        autostart_check.connect_toggled(|btn| {
            let enabled = btn.is_active();
            let _ = autostart::set_enabled(enabled);
        });
        content.append(&autostart_check);

        let notif_check = CheckButton::with_label(&l10n.string("settings.notifications", &[]));
        notif_check.set_active(settings.notifications_enabled());
        let settings_clone3 = settings.clone();
        notif_check.connect_toggled(move |btn| {
            settings_clone3.set_notifications_enabled(btn.is_active());
        });
        content.append(&notif_check);

        content.append(&Separator::new(Orientation::Horizontal));

        let version_label = Label::new(Some(&format!("HerdrBell v{}", env!("CARGO_PKG_VERSION"))));
        version_label.add_css_class("dim-label");
        content.append(&version_label);

        window.set_child(Some(&content));
        window.present();
    }
}
