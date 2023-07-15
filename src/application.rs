use adw::subclass::prelude::*;
use gio::Settings;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::{cell::OnceCell, sync::Arc};

use crate::config::VERSION;
use crate::widgets::onboarding::OnboardingWindow;
use crate::widgets::window::PryvidWindow;

mod imp {

    use crate::config::APP_ID;

    use super::*;

    #[derive(Debug, Default)]
    pub struct PryvidApplication {
        pub settings: OnceCell<Arc<Settings>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PryvidApplication {
        const NAME: &'static str = "PryvidApplication";
        type Type = super::PryvidApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for PryvidApplication {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
        }
    }

    impl ApplicationImpl for PryvidApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.obj();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let settings = Settings::new(APP_ID);
                if settings.boolean("first-run") {
                    let window = OnboardingWindow::new(&*application);
                    window.upcast()
                } else {
                    let window = PryvidWindow::new(&*application);
                    window.upcast()
                }
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for PryvidApplication {}
    impl AdwApplicationImpl for PryvidApplication {}
}

glib::wrapper! {
    pub struct PryvidApplication(ObjectSubclass<imp::PryvidApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PryvidApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name("pryvid")
            .application_icon("dev.quark97.Pryvid")
            .developer_name("Quark97")
            .version(VERSION)
            .developers(vec!["Quark97"])
            .copyright("© 2023 Quark97")
            .build();

        about.present();
    }
}
