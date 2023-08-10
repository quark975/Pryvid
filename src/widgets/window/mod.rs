use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib::ffi::GVariant;
use gtk::glib::{ControlFlow, MainContext, Priority};
use gtk::prelude::*;
use gtk::{gio, glib};
use std::sync::Arc;
use std::{cell::OnceCell, thread};

use crate::api::Content;
use crate::appmodel::AppModel;
use crate::widgets::content_grid::ContentGrid;

use super::content_grid::ContentGridState;

mod imp {

    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/window.ui")]
    pub struct PryvidWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub popular_grid: TemplateChild<ContentGrid>,
        #[template_child]
        pub trending_grid: TemplateChild<ContentGrid>,

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
        let open_channel_action = gio::ActionEntry::builder("open-channel")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(move |win: &Self, _, param| {
                if let Some(param) = param {
                    let channel_id = param.get::<String>().unwrap();
                    println!("{channel_id}");
                }
            })
            .build();

        let open_video_action = gio::ActionEntry::builder("open-video")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(move |win: &Self, _, param| {
                if let Some(param) = param {
                    let video_id = param.get::<String>().unwrap();
                    println!("{video_id}");
                }
            })
            .build();

        self.add_action_entries([notify_action, open_channel_action, open_video_action]);
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn fetch_startup(&self) {
        let imp = self.imp();

        MainContext::default().spawn_local(clone!(@weak self as window => async move {
            let invidious = window.model().invidious();
            let popular = invidious.popular();
            let trending = invidious.trending();

            let imp = window.imp();
            imp.popular_grid.set_state(ContentGridState::Loading);
            imp.trending_grid.set_state(ContentGridState::Loading);

            let (popular, trending) = futures::join!(popular, trending);
            window.imp().popular_grid.set_state(
                match popular {
                    Ok(content) => {
                        if content.len() == 0 {
                            // TODO: Add a way to know which instance made the request
                            ContentGridState::NoContent(("No Popular Videos".into(), "Try closing and reopening the app.".into()))
                        } else {
                            ContentGridState::Success(content)
                        }
                    },
                    Err(error) => {
                        ContentGridState::Error(error.to_string())
                    }
                }
            );
            window.imp().trending_grid.set_state(
                match trending {
                    Ok(content) => {
                        if content.len() == 0 {
                            // TODO: Add a refresh button
                            ContentGridState::NoContent(("No Trending Videos".into(), "Try closing and reopening the app.".into()))
                        } else {
                            ContentGridState::Success(content)
                        }
                    },
                    Err(error) => {
                        ContentGridState::Error(error.to_string())
                    }
                }
            );
        }));
    }
}
