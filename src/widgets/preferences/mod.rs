use adw::subclass::prelude::*;
use gio::Settings;
use gtk::prelude::*;
use gtk::{gio, glib};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/preferences.ui")]
    pub struct PryvidPreferencesWindow {}

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
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}
