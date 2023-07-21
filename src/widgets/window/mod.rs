use crate::appmodel::AppModel;
use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib::{GString, MainContext, Priority};
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
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,

        pub model: OnceCell<Arc<AppModel>>,
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
            self.obj().setup_gactions();
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
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P, model: Arc<AppModel>) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", application)
            .build();

        // Setup window
        window.imp().model.set(model).unwrap();
        window.fetch_startup();

        window
    }

    fn setup_gactions(&self) {
        let notify_action = gio::ActionEntry::builder("notify")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(move |win: &Self, _, param| {
                if let Some(param) = param {
                    if let Some(message) = param.get::<String>() {
                        win.imp()
                            .toast_overlay
                            .add_toast(adw::Toast::builder().title(message).build());
                    }
                }
            })
            .build();

        self.add_action_entries([notify_action]);
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn fetch_startup(&self) {
        let (sender, receiver) = MainContext::channel(Priority::default());
        let invidious = self.model().invidious();
        let imp = self.imp();

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
