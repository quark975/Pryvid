use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, subclass::Signal, Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use std::cell::RefCell;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::ResultPage)]
    #[template(resource = "/dev/quark97/Pryvid/result_page.ui")]
    pub struct ResultPage {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub content_bin: TemplateChild<adw::Bin>,
        #[template_child]
        pub status_page: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub refresh_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,

        #[property(get, set)]
        pub child: RefCell<Option<gtk::Widget>>,
        #[property(get, set)]
        pub refreshable: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResultPage {
        const NAME: &'static str = "ResultPage";
        type Type = super::ResultPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ResultPage {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("refresh").build()]);
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
    impl WidgetImpl for ResultPage {}
    impl BinImpl for ResultPage {}

    #[gtk::template_callbacks]
    impl ResultPage {
        #[template_callback]
        fn on_refresh_button_clicked(&self, _: gtk::Button) {
            self.obj().emit_by_name::<()>("refresh", &[]);
        }
    }
}

glib::wrapper! {
    pub struct ResultPage(ObjectSubclass<imp::ResultPage>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

pub enum ResultPageState {
    Loading,
    Success,
    Message((String, String, String)), // Icon, Title, Description
    Error(String),
}

impl ResultPage {
    pub fn new(child: impl IsA<gtk::Widget>) -> Self {
        Object::builder().property("child", &child).build()
    }

    pub fn set_state(&self, state: ResultPageState) {
        self.imp().stack.set_visible_child_name(match state {
            ResultPageState::Loading => {
                self.imp().spinner.start();
                self.imp().spinner.set_spinning(true);
                "loading"
            }
            ResultPageState::Success => {
                self.imp().spinner.stop();
                self.imp().spinner.set_spinning(false);
                "content"
            }
            ResultPageState::Message((icon, title, description)) => {
                let status_page = &self.imp().status_page;
                status_page.set_icon_name(Some(&icon));
                status_page.set_title(&title);
                status_page.set_description(Some(&description));
                self.imp().spinner.stop();
                self.imp().spinner.set_spinning(false);
                "status"
            }
            ResultPageState::Error(message) => {
                let status_page = &self.imp().status_page;
                status_page.set_icon_name(Some("dialog-error-symbolic"));
                status_page.set_title("An Error Occurred");
                status_page.set_description(Some(&message));
                self.imp().spinner.stop();
                self.imp().spinner.set_spinning(false);
                "status"
            }
        });
    }
}
