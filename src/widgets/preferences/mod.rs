use adw::subclass::prelude::*;
use adw::traits::ActionRowExt;
use gio::Settings;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::{cell::OnceCell, sync::Arc};

use crate::appmodel::AppModel;

mod imp {


    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/preferences.ui")]
    pub struct PryvidPreferencesWindow {
        #[template_child]
        pub instances_listbox: TemplateChild<gtk::ListBox>,
        pub model: OnceCell<Arc<AppModel>>
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
        window.setup();
        window
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn setup(&self) {
        let invidious = self.model().invidious();
        let instances = invidious.instances();
        println!("{:?}", instances);
        for instance in instances {
            let row = adw::ActionRow::builder()
                .title(&instance.uri)
                .build();
            self.imp().instances_listbox.append(&row)
        }
    }
}
