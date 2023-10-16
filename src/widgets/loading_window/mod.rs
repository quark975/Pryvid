use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{subclass::Signal, Object};
use gtk::glib;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

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
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LoadingWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("canceled").build()]);
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for LoadingWindow {}
    impl WindowImpl for LoadingWindow {}
    impl AdwWindowImpl for LoadingWindow {}

    #[gtk::template_callbacks]
    impl LoadingWindow {
        #[template_callback]
        fn on_cancel_clicked(&self) {
            self.obj().emit_by_name::<()>("canceled", &[]);
        }
    }
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
