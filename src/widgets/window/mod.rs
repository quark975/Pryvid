use crate::api::{fetch_instances, Instance, Instances, InvidiousClient};
use crate::application::PryvidApplication;
use crate::config::APP_ID;
use adw::subclass::prelude::*;
use gio::Settings;
use glib::clone;
use gtk::glib::{BoolError, MainContext, Priority};
use gtk::prelude::*;
use gtk::{gio, glib};
use std::sync::Arc;
use std::{cell::OnceCell, thread};

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/window.ui")]
    pub struct PryvidWindow {
        // Template widgets
        #[template_child]
        pub header_bar: TemplateChild<gtk::HeaderBar>,
        #[template_child]
        pub label: TemplateChild<gtk::EditableLabel>,

        pub invidious: OnceCell<Arc<InvidiousClient>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PryvidWindow {
        const NAME: &'static str = "PryvidWindow";
        type Type = super::PryvidWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PryvidWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_invidious();
            obj.fetch_startup();
        }
    }
    impl WidgetImpl for PryvidWindow {}
    impl WindowImpl for PryvidWindow {}
    impl ApplicationWindowImpl for PryvidWindow {}
    impl AdwApplicationWindowImpl for PryvidWindow {}
}

glib::wrapper! {
    pub struct PryvidWindow(ObjectSubclass<imp::PryvidWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl PryvidWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_invidious(&self) {
        // TODO: Handle errors a little better
        let instances = self.load_instances().unwrap();
        let invidious = Arc::new(InvidiousClient::new(instances));
        match self.imp().invidious.set(invidious) {
            Err(_) => panic!("`invidious` should not be set before calling `setup_invidious`"),
            _ => (),
        }
    }

    fn invidious(&self) -> Arc<InvidiousClient> {
        self.imp()
            .invidious
            .get()
            .expect("`invidious` should be set with `setup_invidious`")
            .clone()
    }

    fn save_instances(&self) -> Result<(), BoolError> {
        let settings = Settings::new(APP_ID);
        let invidious = self.invidious();
        let instances = invidious.instances.read().unwrap();
        Ok(settings.set(
            "instances",
            serde_json::to_string(&instances.to_vec())
                .unwrap()
                .to_string(),
        )?)
    }

    fn load_instances(&self) -> Result<Instances, serde_json::Error> {
        let settings = Settings::new(APP_ID);
        let instances: String = settings.get("instances");
        Ok(serde_json::from_str(&settings.string("instances"))?)
    }

    fn fetch_startup(&self) {
        let (sender, receiver) = MainContext::channel(Priority::default());
        let invidious = self.invidious();
        let imp = self.imp();

        {
            let instances = invidious.instances.read().unwrap();
            let instance = instances.get(0).unwrap();

            invidious.select_instance(Some(instance));
        }

        thread::spawn(move || {
            sender
                .send(invidious.stats())
                .expect("Failed to send message.");
        });
        receiver.attach(
            None,
            clone!(@weak imp => @default-return Continue(false),
                move |stats| {
                    imp.label.set_text(&format!("{:?}", stats));
                    Continue(true)
                }
            ),
        );
    }
}
