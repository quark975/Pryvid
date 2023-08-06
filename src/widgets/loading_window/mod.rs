use adw::subclass::prelude::*;
use glib::Object;
use gtk::glib;
use gtk::CompositeTemplate;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/loading_window.ui")]
    pub struct LoadingWindow;

    #[glib::object_subclass]
    impl ObjectSubclass for LoadingWindow {
        const NAME: &'static str = "LoadingWindow";
        type Type = super::LoadingWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LoadingWindow {}
    impl WidgetImpl for LoadingWindow {}
    impl WindowImpl for LoadingWindow {}
    impl AdwWindowImpl for LoadingWindow {}
}


glib::wrapper! {
    pub struct LoadingWindow(ObjectSubclass<imp::LoadingWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Native, gtk::Buildable, gtk::ConstraintTarget, gtk::Root, gtk::ShortcutManager;
}

impl LoadingWindow {
    pub fn new() -> Self {
        Object::builder().build()
    }
}