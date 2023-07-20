use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::prelude::*;
use adw::ResponseAppearance;
use gio::Settings;
use gtk::glib::{closure_local, clone, closure};
use gtk::{gio, glib};
use std::{cell::OnceCell, sync::Arc};

use crate::appmodel::AppModel;
use crate::widgets::instancerow::InstanceRow;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/preferences.ui")]
    pub struct PryvidPreferencesWindow {
        #[template_child]
        pub instances_listbox: TemplateChild<gtk::ListBox>,
        pub model: OnceCell<Arc<AppModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PryvidPreferencesWindow {
        const NAME: &'static str = "PryvidPreferencesWindow";
        type Type = super::PryvidPreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PryvidPreferencesWindow {}
    impl WidgetImpl for PryvidPreferencesWindow {}
    impl WindowImpl for PryvidPreferencesWindow {}
    impl AdwWindowImpl for PryvidPreferencesWindow {}
    impl PreferencesWindowImpl for PryvidPreferencesWindow {}
}

glib::wrapper! {
    pub struct PryvidPreferencesWindow(ObjectSubclass<imp::PryvidPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gtk::ShortcutManager, gtk::Root, gtk::Native, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl PryvidPreferencesWindow {
    pub fn new(model: Arc<AppModel>) -> Self {
        let window: Self = glib::Object::builder().build();
        window.imp().model.set(model).unwrap();
        window.build();
        window
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn rebuild(&self) {
        let listbox = &self.imp().instances_listbox;
        loop {
            let child = listbox.row_at_index(0);
            if let Some(child) = child {
                listbox.remove(&child)
            } else {
                break
            }
        }
        self.build()
    }

    fn build(&self) {
        let invidious = self.model().invidious();
        let instances = invidious.instances();
        invidious.select_instance(instances.get(0));
        for instance in instances {
            let row = InstanceRow::new(instance);
            row.connect_closure("delete", false, closure_local!(@watch self as window => move |row: InstanceRow| {
                let dialog = adw::MessageDialog::builder()
                    .heading("Remove instance?")
                    .body(format!("Are you sure you want to delete '{}'?", row.title()))
                    .transient_for(window)
                    .build();
                dialog.add_responses(&[("cancel", "Cancel"), ("delete", "Delete")]);
                dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
                dialog.connect_response(Some("delete"), clone!(@weak window, @weak row => move |_, _| {
                    let instance = row.imp().instance.get().unwrap();
                    window.model().invidious().remove_instance(instance.clone()).unwrap();
                    window.imp().instances_listbox.remove(&row);
                    println!("{:?}", window.model().invidious());
                }));
                dialog.present();
            }));
            self.imp().instances_listbox.append(&row)
        }
    }
}
