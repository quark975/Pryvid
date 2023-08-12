use adw::subclass::prelude::*;
use glib::clone;
use gtk::glib::MainContext;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::cell::OnceCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::api::{Content, Error};
use crate::appmodel::AppModel;
use crate::widgets::content_grid::ContentGrid;
use crate::widgets::instance_indicator::InstanceIndicator;

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
        #[template_child]
        pub popular_instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub trending_instance_indicator: TemplateChild<InstanceIndicator>,

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
        MainContext::default().spawn_local(clone!(@weak self as window => async move {
            let invidious = window.model().invidious();

            let imp = window.imp();
            imp.popular_grid.set_state(ContentGridState::Loading);
            imp.trending_grid.set_state(ContentGridState::Loading);
            let popular_instance = invidious.get_popular_instance();
            let trending_instance = invidious.get_trending_instance();

            // This mess is the best way I could come up with to handle joining the futures while
            // accounting for the possibility the user hadn't configured an instance that supports
            // the popular or trending page
            let (popular, trending) = {
                if let Ok(popular_instance) = popular_instance {
                    if let Ok(trending_instance) = trending_instance {
                        imp.popular_instance_indicator.set_uri(popular_instance.uri.clone());
                        imp.trending_instance_indicator.set_uri(trending_instance.uri.clone());
                        let result = futures::join!(popular_instance.popular(), trending_instance.trending());
                        (Some(result.0), Some(result.1))
                    } else {
                        imp.popular_instance_indicator.set_uri(popular_instance.uri.clone());
                        (Some(popular_instance.popular().await), None)
                    }
                } else if let Ok(trending_instance) = trending_instance {
                        imp.trending_instance_indicator.set_uri(trending_instance.uri.clone());
                    (None, Some(trending_instance.trending().await))
                } else {
                    (None, None)
                }
            };
            window.imp().popular_grid.set_state(
                match popular {
                    Some(Ok(content)) => {
                        if content.len() == 0 {
                            // TODO: Add a way to know which instance made the request
                            ContentGridState::NoContent(("No Popular Videos".into(), "Try closing and reopening the app.".into()))
                        } else {
                            ContentGridState::Success(content)
                        }
                    },
                    Some(Err(error)) => {
                        ContentGridState::Error(error.to_string())
                    },
                    None => {
                        ContentGridState::Error("None of your instances support popular videos.".into())
                    }
                }
            );
            window.imp().trending_grid.set_state(
                match trending {
                    Some(Ok(content)) => {
                        if content.len() == 0 {
                            // TODO: Add a refresh button
                            ContentGridState::NoContent(("No Trending Videos".into(), "Try closing and reopening the app.".into()))
                        } else {
                            ContentGridState::Success(content)
                        }
                    },
                    Some(Err(error)) => {
                        ContentGridState::Error(error.to_string())
                    },
                    None => {
                        ContentGridState::Error("None of your instances support trending videos.".into())
                    }
                }
            );
        }));
    }
}
