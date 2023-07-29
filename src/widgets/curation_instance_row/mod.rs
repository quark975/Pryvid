use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;
use gtk::glib;
use std::cell::OnceCell;
use std::sync::Arc;

use crate::api::Instance;

mod imp {

    use super::*;

    #[derive(Default, Debug)]
    pub struct CurationInstanceRow {
        pub instance: OnceCell<Arc<Instance>>,
        pub ping_label: OnceCell<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CurationInstanceRow {
        const NAME: &'static str = "CurationInstanceRow";
        type Type = super::CurationInstanceRow;
        type ParentType = adw::ExpanderRow;
    }

    impl ObjectImpl for CurationInstanceRow {}
    impl WidgetImpl for CurationInstanceRow {}
    impl ListBoxRowImpl for CurationInstanceRow {}
    impl PreferencesRowImpl for CurationInstanceRow {}
    impl ExpanderRowImpl for CurationInstanceRow {}
}

glib::wrapper! {
    pub struct CurationInstanceRow(ObjectSubclass<imp::CurationInstanceRow>)
        @extends adw::ExpanderRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

pub enum PingState {
    NotPinged,
    Pinging,
    Success(u128),
    Error
}

impl CurationInstanceRow {
    pub fn new(instance: Arc<Instance>) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().instance.set(instance).unwrap();
        obj.build();
        obj
    }

    pub fn set_state(&self, state: PingState) {
        let label = self.ping_label();
        match state {
            PingState::NotPinged => label.set_text(""),
            PingState::Pinging => {
                label.set_text("Pinging...");
                label.set_css_classes(&["accent"]);
            },
            PingState::Success(ping) => {
                label.set_text(&format!("{} ms", ping));
                label.set_css_classes(if ping < 1000 {
                    &["success"]
                } else if ping < 2000 {
                    &["warning"]
                } else {
                    &["error"]
                })
            },
            PingState::Error => {
                label.set_text("Failed");
                label.set_css_classes(&["error"]);
            }
        }
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.imp().instance.get().unwrap().clone()
    }

    fn ping_label(&self) -> gtk::Label {
        self.imp().ping_label.get().unwrap().clone()
    }

    fn add_data(&self, title: &str, subtitle: &str) {
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();
        self.add_row(&row);
    }

    fn build(&self) {
        let instance = self.instance();

        // Build widget
        let label = gtk::Label::new(None);
        self.add_suffix(&label);
        self.imp().ping_label.set(label).unwrap();

        self.set_title(&instance.uri);
        self.add_data(
            "Popular Tab",
            if instance.has_popular {
                "Available"
            } else {
                "Unavailable"
            },
        );
        self.add_data(
            "Trending Tab",
            if instance.has_trending {
                "Available"
            } else {
                "Unavailable"
            },
        );
        self.add_data(
            "Registrations",
            if instance.open_registrations {
                "Open"
            } else {
                "Closed"
            },
        );
    }
}
