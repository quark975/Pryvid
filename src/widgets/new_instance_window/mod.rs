use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass::Signal, MainContext, Object};
use gtk::glib;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use std::cell::OnceCell;
use std::sync::Arc;

use crate::api::{Error, Instance};
use crate::appmodel::AppModel;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/new_instance_window.ui")]
    pub struct NewInstanceWindow {
        pub model: OnceCell<Arc<AppModel>>,

        #[template_child]
        pub instance_entry: gtk::TemplateChild<adw::EntryRow>,
        #[template_child]
        pub create_button: gtk::TemplateChild<gtk::Button>,
        #[template_child]
        pub error_label: gtk::TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewInstanceWindow {
        const NAME: &'static str = "NewInstanceWindow";
        type Type = super::NewInstanceWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NewInstanceWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("added-instance").build()]);
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for NewInstanceWindow {}
    impl WindowImpl for NewInstanceWindow {}
    impl AdwWindowImpl for NewInstanceWindow {}

    #[gtk::template_callbacks]
    impl NewInstanceWindow {
        #[template_callback]
        fn on_create_clicked(&self, _: gtk::Button) {
            let text = self.instance_entry.text();
            if text.len() > 0 {
                self.instance_entry.set_sensitive(false);
                self.create_button.set_sensitive(false);

                MainContext::default().spawn_local(clone!(@weak self as window, @strong text => async move {
                    let instance_result = Instance::from_uri(&text).await;

                    match instance_result {
                        Ok(instance) => {
                            match window.obj().model().invidious().push_instance(Arc::new(instance)) {
                                Ok(_) => window.obj().emit_by_name::<()>("added-instance", &[]),
                                Err(_) => window.obj().display_error("Instance is already added!"),
                            }
                        },
                        Err(err) => {
                            let response = match err {
                                Error::DeserializeError(_) => "DeserializeError: Likely not an Invidious server.".into(),
                                _ => err.to_string()
                            };
                            window.obj().display_error(&response);
                        },
                    }
                    window.instance_entry.set_sensitive(true);
                    window.create_button.set_sensitive(true);
                }));
            }
        }
    }
}

glib::wrapper! {
    pub struct NewInstanceWindow(ObjectSubclass<imp::NewInstanceWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Native, gtk::Buildable, gtk::ConstraintTarget, gtk::Root, gtk::ShortcutManager;
}

impl NewInstanceWindow {
    pub fn new(model: Arc<AppModel>) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().model.set(model).unwrap();
        obj
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn display_error(&self, message: &str) {
        let imp = self.imp();
        imp.instance_entry.set_css_classes(&["error"]);
        imp.error_label.set_text(message);
    }
}
