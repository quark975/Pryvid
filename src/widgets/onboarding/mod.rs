use adw::subclass::prelude::*;
use gio::Settings;
use glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::widgets::PryvidWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/onboarding.ui")]
    pub struct OnboardingWindow {
        #[template_child]
        pub getstarted: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OnboardingWindow {
        const NAME: &'static str = "OnboardingWindow";
        type Type = super::OnboardingWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for OnboardingWindow {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.setup_callbacks();
        }
    }
    impl WidgetImpl for OnboardingWindow {}
    impl WindowImpl for OnboardingWindow {}
    impl ApplicationWindowImpl for OnboardingWindow {}
    impl AdwApplicationWindowImpl for OnboardingWindow {}
}

glib::wrapper! {
    pub struct OnboardingWindow(ObjectSubclass<imp::OnboardingWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl OnboardingWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();
        imp.getstarted
            .connect_clicked(clone!(@weak self as _self => move |_| {
                let application = _self.application().unwrap();
                let app_id = application.application_id().unwrap();
                // TODO: Could store settings on application, minor optimization
                let settings = Settings::new(&app_id);
                settings.set_boolean("first-run", false).expect("Could not set setting.");

                let window = PryvidWindow::new(&application);
                _self.close();
                window.present();
            }));
    }
}
