use gio::prelude::*;

pub struct Notifier;

impl Notifier {
    pub fn new() -> Self {
        Self
    }

    pub fn post(&self, title: &str, body: &str) {
        let app = gio::Application::default();
        if let Some(app) = app {
            let notification = gio::Notification::new(title);
            notification.set_body(Some(body));
            notification.set_priority(gio::NotificationPriority::Normal);
            app.send_notification(Some("herdrbell-agent"), &notification);
        } else {
            let notification = gio::Notification::new(title);
            notification.set_body(Some(body));
            log::warn!("no GApplication available, notification may not display");
        }
    }
}
