use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::ResponseAppearance;
use glib::{clone, Object};
use gtk::glib;
use gtk::glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::Align;
use std::cell::OnceCell;
use std::sync::Arc;

use crate::api::Instance;

mod imp {

    use super::*;

    #[derive(Default, Debug)]
    pub struct InstanceRow {
        pub instance: OnceCell<Arc<Instance>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InstanceRow {
        const NAME: &'static str = "InstanceRow";
        type Type = super::InstanceRow;
        type ParentType = adw::ExpanderRow;
    }

    impl ObjectImpl for InstanceRow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("delete").build()]);
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for InstanceRow {}
    impl ListBoxRowImpl for InstanceRow {}
    impl PreferencesRowImpl for InstanceRow {}
    impl ExpanderRowImpl for InstanceRow {}
}

glib::wrapper! {
    pub struct InstanceRow(ObjectSubclass<imp::InstanceRow>)
        @extends adw::ExpanderRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl InstanceRow {
    pub fn new(instance: Arc<Instance>) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().instance.set(instance).unwrap();
        obj.setup();
        obj
    }

    fn setup(&self) {
        let instance = self.instance();
        self.set_title(&instance.uri);

        // Add info row
        let row = adw::ActionRow::builder()
            .title("Registrations")
            .subtitle(if instance.open_registrations {
                "Open"
            } else {
                "Closed"
            })
            .build();
        self.add_row(&row);

        // Create delete button
        let delete_button = gtk::Button::builder()
            .label("Delete")
            .css_classes(["destructive-action"])
            .vexpand(false)
            .valign(Align::Center)
            .build();
        delete_button.connect_clicked(clone!(@weak self as obj => move |_| {
            obj.emit_by_name::<()>("delete", &[]);
        }));

        // Add buttons row
        let row = adw::ActionRow::new();
        row.add_suffix(&delete_button);
        self.add_row(&row);
    }

    fn instance(&self) -> Arc<Instance> {
        self.imp().instance.get().unwrap().clone()
    }
}
