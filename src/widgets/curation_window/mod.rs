use adw::subclass::prelude::*;
use glib::Object;
use gtk::glib;
use gtk::CompositeTemplate;
use std::sync::Arc;
use std::cell::OnceCell;
use crate::appmodel::AppModel;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/curation_window.ui")]
    pub struct CurationWindow {
        pub model: OnceCell<Arc<AppModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CurationWindow {
        const NAME: &'static str = "CurationWindow";
        type Type = super::CurationWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CurationWindow {}
    impl WidgetImpl for CurationWindow {}
    impl WindowImpl for CurationWindow {}
    impl AdwWindowImpl for CurationWindow {}
}


glib::wrapper! {
    pub struct CurationWindow(ObjectSubclass<imp::CurationWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Native, gtk::Buildable, gtk::ConstraintTarget, gtk::Root, gtk::ShortcutManager;
}

impl CurationWindow {
    pub fn new(model: Arc<AppModel>) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().model.set(model).unwrap();
        obj
    }
}
