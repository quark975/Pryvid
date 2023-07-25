use adw::subclass::prelude::*;
use gtk::prelude::*;
use glib::Object;
use gtk::glib;
use gtk::CompositeTemplate;
use glib::{clone, MainContext, Priority};
use std::cell::OnceCell;
use std::sync::Arc;
use std::thread;

use crate::appmodel::AppModel;
use crate::api::{Instance, Error};

mod imp {

    use gtk::glib::subclass::Signal;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/new_instance_window.ui")]
    pub struct NewInstanceWindow {
        pub model: OnceCell<Arc<AppModel>>,

        #[template_child]
        pub instance_entry: gtk::TemplateChild<adw::EntryRow>,
        #[template_child]
        pub create_button: gtk::TemplateChild<gtk::Button>
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
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("added-instance")
                        .build()
                ]
            });
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

                let (sender, receiver) = MainContext::channel(Priority::default());
                thread::spawn(move || {
                    sender
                        .send(Instance::from_uri(&text))
                        .expect("Failed to send message.");
                });
                receiver.attach(None, clone!(@weak self as window => @default-return Continue(false),
                    move |result: Result<Instance, Error>| {
                        match result {
                            Ok(instance) => {
                                window.obj().model().invidious().push_instance(instance).unwrap();
                                window.obj().emit_by_name::<()>("added-instance", &[]);
                            },
                            Err(err) => {
                                // TODO: Actually handle errors
                                println!("{:?}", err);
                            }
                        }
                        window.instance_entry.set_sensitive(true);
                        window.create_button.set_sensitive(true);

                        Continue(false)
                    }
                ));
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
}
