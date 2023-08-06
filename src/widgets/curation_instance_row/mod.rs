use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;
use gtk::{glib, Align};
use glib::clone;
use glib::subclass::Signal;
use std::cell::{OnceCell, Cell};
use std::sync::Arc;
use once_cell::sync::Lazy;

use crate::api::Instance;

mod imp {

    use super::*;

    #[derive(Default, Debug)]
    pub struct CurationInstanceRow {
        pub instance: OnceCell<Arc<Instance>>,
        pub ping_label: OnceCell<gtk::Label>,
        pub add_button: OnceCell<gtk::Button>,
        pub ping_state: Cell<PingState>,
        pub added: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CurationInstanceRow {
        const NAME: &'static str = "CurationInstanceRow";
        type Type = super::CurationInstanceRow;
        type ParentType = adw::ExpanderRow;
    }

    impl ObjectImpl for CurationInstanceRow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("toggle")
                        .build()
                ]
            });
            SIGNALS.as_ref()
        }
    }
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

#[derive(Debug, Clone, Copy)]
pub enum PingState {
    NotPinged,
    Pinging,
    Success(u128),
    Error
}

impl Default for PingState {
    fn default() -> Self {
        PingState::NotPinged
    }
}

impl CurationInstanceRow {
    pub fn new(instance: Arc<Instance>, is_added: bool) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().instance.set(instance).unwrap();
        obj.set_added(is_added);
        obj.build();
        obj
    }

    pub fn ping_state(&self) -> PingState {
        self.imp().ping_state.get()
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.imp().instance.get().unwrap().clone()
    }

    pub fn set_state(&self, state: PingState) {
        self.imp().ping_state.set(state.clone());
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

    pub fn set_add_button_visible(&self, is_visible: bool) {
        self.add_button().set_visible(is_visible)
    }

    pub fn added(&self) -> bool {
        self.imp().added.get()
    }

    fn ping_label(&self) -> gtk::Label {
        self.imp().ping_label.get().unwrap().clone()
    }

    fn add_button(&self) -> gtk::Button {
        self.imp().add_button.get().unwrap().clone()
    }
    

    fn set_added(&self, added: bool) {
        self.imp().added.set(added)
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
        let button = gtk::Button::builder()
            .icon_name(if self.added() {
                "list-remove-symbolic"
            } else {
                "list-add-symbolic"
            })
            .vexpand(false)
            .valign(Align::Center)
            .visible(false)
            .css_classes(["flat"])
            .build();
        button.connect_clicked(clone!(@weak self as window => move |button| {
            if window.added() {
                button.set_icon_name("list-add-symbolic");
                window.set_added(false);
            } else {
                button.set_icon_name("list-remove-symbolic");
                window.set_added(true);
            }
            window.emit_by_name::<()>("toggle", &[]);
        }));
        self.add_prefix(&button);
        self.imp().add_button.set(button).unwrap();

        let label = gtk::Label::new(None);
        self.add_suffix(&label);
        self.imp().ping_label.set(label).unwrap();

        self.set_title(&instance.uri);

        let info = instance.info.read().unwrap();
        self.add_data(
            "Popular Tab",
            if info.has_popular {
                "Available"
            } else {
                "Unavailable"
            },
        );
        self.add_data(
            "Trending Tab",
            if info.has_trending {
                "Available"
            } else {
                "Unavailable"
            },
        );
        self.add_data(
            "Registrations",
            if info.open_registrations {
                "Open"
            } else {
                "Closed"
            },
        );
    }
}
