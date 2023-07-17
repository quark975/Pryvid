use adw::subclass::prelude::*;
use gio::Settings;
use glib::clone;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::{cell::OnceCell, sync::Arc};

use crate::api::InvidiousClient;
use crate::appmodel::AppModel;
use crate::config::APP_ID;
use crate::widgets::window::PryvidWindow;

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/onboarding.ui")]
    pub struct OnboardingWindow {
        #[template_child]
        pub getstarted: TemplateChild<gtk::Button>,

        pub model: OnceCell<Arc<AppModel>>,
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
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P, model: Arc<AppModel>) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", application)
            .build();
        window.imp().model.set(model);
        window
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn setup_callbacks(&self) {
        let imp = self.imp();
        // imp.getstarted
        //     .connect_clicked(clone!(@weak self as _self => move |_| {
        //         // TODO: Could store settings on application, minor optimization
        //         let settings = Settings::new(APP_ID);
        //         settings.set_boolean("first-run", false).expect("Could not set setting.");
        //
        //         let window = PryvidWindow::new(&_self.application().unwrap().clone(), self.model());
        //         _self.close();
        //         window.present();
        //     }));
    }
}
