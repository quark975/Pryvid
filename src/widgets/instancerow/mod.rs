use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::ResponseAppearance;
use gtk::glib::{once_cell::sync::Lazy, subclass::Signal};
use glib::{clone, Object};
use gtk::glib;
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
        type ParentType = adw::ActionRow;
    }

    impl ObjectImpl for InstanceRow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("delete")
                        .build()
                ]
            });
            SIGNALS.as_ref()
        }
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Create delete button
            let delete_button = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .css_classes(["destructive-action"])
                .vexpand(false)
                .valign(Align::Center)
                .build();
            delete_button.connect_clicked(clone!(@weak obj => move |_| {
                obj.emit_by_name::<()>("delete", &[]);
            }));

            obj.add_suffix(&delete_button);
        }
    }
    impl WidgetImpl for InstanceRow {}
    impl ListBoxRowImpl for InstanceRow {}
    impl PreferencesRowImpl for InstanceRow {}
    impl ActionRowImpl for InstanceRow {}
}

glib::wrapper! {
    pub struct InstanceRow(ObjectSubclass<imp::InstanceRow>)
        @extends adw::ActionRow, adw::PreferencesRow, gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl InstanceRow {
    pub fn new(instance: Arc<Instance>) -> Self {
        let obj: Self = Object::builder().build();
        obj.set_title(&instance.uri);

        obj.imp().instance.set(instance).unwrap();

        obj
    }
}
