use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass::Signal, Object, Properties};
use gtk::{glib, Align};
use once_cell::sync::Lazy;
use std::cell::{Cell, OnceCell};
use std::sync::Arc;

use crate::api::Instance;

mod imp {

    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type = super::CurationInstanceRow)]
    pub struct CurationInstanceRow {
        pub instance: OnceCell<Arc<Instance>>,
        pub ping_label: OnceCell<gtk::Label>,
        pub add_button: OnceCell<gtk::Button>,
        pub ping_state: Cell<PingState>,
        #[property(get, set = Self::set_added)]
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
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("toggle").build()]);
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
    impl WidgetImpl for CurationInstanceRow {}
    impl ListBoxRowImpl for CurationInstanceRow {}
    impl PreferencesRowImpl for CurationInstanceRow {}
    impl ExpanderRowImpl for CurationInstanceRow {}

    impl CurationInstanceRow {
        fn set_added(&self, added: bool) {
            self.added.set(added);
            let button = self.add_button.get().unwrap();
            if added {
                button.set_icon_name("list-remove-symbolic");
            } else {
                button.set_icon_name("list-add-symbolic");
            }
        }
    }
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
    Error,
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
        obj.build();
        obj.set_added(is_added);
        obj
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.imp().instance.get().unwrap().clone()
    }

    pub fn ping_state(&self) -> PingState {
        self.imp().ping_state.get()
    }

    pub fn set_ping_state(&self, ping_state: PingState) {
        self.imp().ping_state.set(ping_state);
        let label = self.ping_label();
        match ping_state {
            PingState::NotPinged => label.set_text(""),
            PingState::Pinging => {
                label.set_text("Pinging...");
                label.set_css_classes(&["accent"]);
            }
            PingState::Success(ping) => {
                label.set_text(&format!("{} ms", ping));
                label.set_css_classes(if ping < 1000 {
                    &["success"]
                } else if ping < 2000 {
                    &["warning"]
                } else {
                    &["error"]
                });
            }
            PingState::Error => {
                label.set_text("Failed");
                label.set_css_classes(&["error"]);

                let _button = self.add_button();
            }
        }
    }

    fn ping_label(&self) -> gtk::Label {
        self.imp().ping_label.get().unwrap().clone()
    }

    fn add_button(&self) -> gtk::Button {
        self.imp().add_button.get().unwrap().clone()
    }

    fn add_info_row(&self, title: &str, subtitle: &str) {
        let row = adw::ActionRow::builder()
            .title(title)
            .subtitle(subtitle)
            .build();
        self.add_row(&row);
    }

    fn populate_info(&self) {
        let instance = self.instance();

        let info = instance.info.read().unwrap();
        self.add_info_row(
            "Popular Tab",
            if let Some(has_popular) = info.has_popular {
                if has_popular {
                    "Available"
                } else {
                    "Unavailable"
                }
            } else {
                "Unknown"
            },
        );
        self.add_info_row(
            "Trending Tab",
            if let Some(has_trending) = info.has_trending {
                if has_trending {
                    "Available"
                } else {
                    "Unavailable"
                }
            } else {
                "Unknown"
            },
        );
        self.add_info_row(
            "Registrations",
            if info.open_registrations {
                "Open"
            } else {
                "Closed"
            },
        );
    }

    fn build(&self) {
        let instance = self.instance();

        let button = gtk::Button::builder()
            .icon_name(if self.added() {
                "list-remove-symbolic"
            } else {
                "list-add-symbolic"
            })
            .vexpand(false)
            .valign(Align::Center)
            .css_classes(["flat"])
            .build();
        button.connect_clicked(clone!(@weak self as window => move |_| {
            window.emit_by_name::<()>("toggle", &[]);
        }));
        self.add_prefix(&button);
        self.imp().add_button.set(button).unwrap();

        let label = gtk::Label::new(None);
        self.add_suffix(&label);
        self.imp().ping_label.set(label).unwrap();

        self.set_title(&instance.uri);

        self.populate_info();
    }
}
