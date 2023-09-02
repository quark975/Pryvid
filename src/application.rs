use adw::subclass::prelude::*;
use gio::Settings;
use gtk::glib::BoolError;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::{cell::OnceCell, sync::Arc};

use crate::api::{Instances, InvidiousClient};
use crate::appmodel::AppModel;
use crate::config::{APP_ID, VERSION};
use crate::widgets::onboarding::OnboardingWindow;
use crate::widgets::preferences::PryvidPreferencesWindow;
use crate::widgets::window::PryvidWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct PryvidApplication {
        pub model: OnceCell<Arc<AppModel>>,
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
            obj.setup_model();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("win.toggle-fullscreen", &["f"]);
            obj.set_accels_for_action("win.escape-pressed", &["Escape"]);
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
                let settings = application.model().settings();
                if settings.boolean("first-run") {
                    let window = OnboardingWindow::new(&*application, self.obj().model());
                    window.upcast()
                } else {
                    let window = PryvidWindow::new(&*application, self.obj().model());
                    window.upcast()
                }
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }

        fn shutdown(&self) {
            self.parent_shutdown();
            self.obj().save_instances().unwrap();
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

    fn model(&self) -> Arc<AppModel> {
        self.imp()
            .model
            .get()
            .expect("`model` should be set by calling `setup_model`")
            .clone()
    }

    fn setup_model(&self) {
        // Setup Invidious
        // TODO: Handle errors a little better
        let settings = Settings::new(APP_ID);
        let instances = self.load_instances(&settings).unwrap();
        let invidious = InvidiousClient::new(instances);
        invidious
            .select_instance_by_name(settings.string("selected").as_str())
            .unwrap();
        let model = Arc::new(AppModel::new(invidious, settings));
        match self.imp().model.set(model) {
            Err(_) => panic!("`model` should not be set before calling `setup_model`"),
            _ => (),
        }
    }

    fn save_instances(&self) -> Result<(), BoolError> {
        let invidious = self.model().invidious();
        let settings = self.model().settings();
        let instances = invidious.instances();
        settings.set(
            "instances",
            serde_json::to_string(&instances.to_vec())
                .unwrap()
                .to_string(),
        )?;
        settings.set(
            "selected",
            if let Some(instance) = invidious.selected_instance() {
                instance.uri.clone()
            } else {
                "".into()
            },
        )?;
        Ok(())
    }

    fn load_instances(&self, settings: &Settings) -> Result<Instances, serde_json::Error> {
        // --- Only used in testing
        settings.set_boolean("first-run", true).unwrap();
        // settings.reset("instances");
        // ---

        Ok(serde_json::from_str(&settings.string("instances"))?)
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let preferences_action = gio::ActionEntry::builder("preferences")
            .activate(move |app: &Self, _, _| {
                let pref_window = PryvidPreferencesWindow::new(app.model());
                if let Some(main_window) = app.active_window() {
                    pref_window.set_transient_for(Some(&main_window));
                }
                pref_window.present();
            })
            .build();
        let getstarted_action = gio::ActionEntry::builder("getstarted")
            .activate(move |app: &Self, _, _| {
                if let Some(active_window) = app.active_window() {
                    active_window.close()
                }
                let window = PryvidWindow::new(&*app, app.model());
                window.present();
            })
            .build();
        self.add_action_entries([
            quit_action,
            about_action,
            preferences_action,
            getstarted_action,
        ]);
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
