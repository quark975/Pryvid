use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass::Signal, Object, Properties};
use gtk::glib;
use gtk::Align;
use once_cell::sync::Lazy;
use std::cell::{Cell, OnceCell};
use std::sync::Arc;

use crate::api::Instance;

mod imp {

    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type = super::InstanceRow)]
    pub struct InstanceRow {
        pub instance: OnceCell<Arc<Instance>>,
        #[property(get, set)]
        selected: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InstanceRow {
        const NAME: &'static str = "InstanceRow";
        type Type = super::InstanceRow;
        type ParentType = adw::ExpanderRow;
    }

    impl ObjectImpl for InstanceRow {
        fn constructed(&self) {
            self.parent_constructed();

            let switch = gtk::Switch::builder()
                .vexpand(false)
                .valign(Align::Center)
                .build();
            let obj = self.obj();
            obj.add_suffix(&switch);
            obj.bind_property("selected", &switch, "active")
                .bidirectional()
                .sync_create()
                .build();
        }

        // TODO: All of the property-related function will eventually
        // be added via a #[glib::derived_properties] but it isn't released yet
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("delete").build(),
                    Signal::builder("select").build(),
                ]
            });
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
    pub fn new(instance: Arc<Instance>, selected: bool) -> Self {
        let obj: Self = Object::builder().property("selected", selected).build();
        obj.imp().instance.set(instance).unwrap();
        obj.build();
        obj
    }

    pub fn instance(&self) -> Arc<Instance> {
        self.imp().instance.get().unwrap().clone()
    }

    fn build(&self) {
        let instance = self.instance();
        self.set_title(&instance.uri);

        // Add info row
        let row = adw::ActionRow::builder()
            .title("Registrations")
            .subtitle(if instance.info.read().unwrap().open_registrations {
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

}
