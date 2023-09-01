use adw::subclass::prelude::*;
use gdk::prelude::*;
use glib::clone;
use gtk::glib::{closure_local, MainContext};
use gtk::prelude::*;
use gtk::template_callbacks;
use gtk::{gdk, gio, glib};
use std::cell::OnceCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::api::{Content, Error};
use crate::appmodel::AppModel;
use crate::widgets::content_grid::ContentGrid;
use crate::widgets::instance_indicator::InstanceIndicator;
use crate::widgets::result_page::ResultPageState;
use crate::widgets::video_view::VideoView;

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
        pub search_grid: TemplateChild<ContentGrid>,
        #[template_child]
        pub popular_instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub trending_instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub search_instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub navigation_view: TemplateChild<adw::NavigationView>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,

        pub model: OnceCell<Arc<AppModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PryvidWindow {
        const NAME: &'static str = "PryvidWindow";
        type Type = super::PryvidWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PryvidWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().setup_gactions();
            self.obj().setup_callbacks();
        }
    }
    impl WidgetImpl for PryvidWindow {}
    impl WindowImpl for PryvidWindow {}
    impl ApplicationWindowImpl for PryvidWindow {}
    impl AdwApplicationWindowImpl for PryvidWindow {}

    #[template_callbacks]
    impl PryvidWindow {
        #[template_callback]
        fn on_search_entry_search_changed(&self) {
            let search = self.search_entry.text();
            let view_stack = self.view_stack.clone();
            if search.len() == 0 {
                view_stack.set_visible_child_name("popular");
            } else {
                let obj = self.obj();
                MainContext::default().spawn_local(
                    clone!(@weak obj, @weak view_stack => async move {
                        obj.build_search(&search).await;
                        view_stack.set_visible_child_name("search");
                    }),
                );
            }
        }
    }
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
                    let video_view = VideoView::new(win.model(), video_id);
                    win.bind_property::<VideoView>(
                        "fullscreened",
                        video_view.as_ref(),
                        "fullscreened",
                    )
                    .sync_create()
                    .bidirectional()
                    .build();
                    win.imp().navigation_view.push(&video_view);
                }
            })
            .build();

        let fullscreen_action = gio::ActionEntry::builder("fullscreen")
            .parameter_type(None)
            .activate(move |win: &Self, _, param| {
                win.fullscreen();
            })
            .build();
        let unfullscreen_action = gio::ActionEntry::builder("unfullscreen")
            .parameter_type(None)
            .activate(move |win: &Self, _, param| {
                win.unfullscreen();
            })
            .build();
        let toggle_fullscreen_action = gio::ActionEntry::builder("toggle-fullscreen")
            .parameter_type(None)
            .activate(move |win: &Self, _, param| {
                // Only fullscreen if watching a video
                if win
                    .imp()
                    .navigation_view
                    .visible_page()
                    .unwrap()
                    .downcast::<VideoView>()
                    .is_ok()
                {
                    win.set_fullscreened(!win.is_fullscreened())
                }
            })
            .build();

        self.add_action_entries([
            notify_action,
            open_channel_action,
            open_video_action,
            fullscreen_action,
            unfullscreen_action,
            toggle_fullscreen_action,
        ]);
    }

    fn setup_callbacks(&self) {
        self.imp().popular_grid.connect_closure(
            "refresh",
            false,
            closure_local!(@watch self as window => move |_: ContentGrid| {
                MainContext::default().spawn_local(clone!(@weak window => async move {
                    window.build_popular().await;
                }));
            }),
        );
        self.imp().trending_grid.connect_closure(
            "refresh",
            false,
            closure_local!(@watch self as window => move |_: ContentGrid| {
                MainContext::default().spawn_local(clone!(@weak window => async move {
                    window.build_trending().await;
                }));
            }),
        );
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    async fn build_search(&self, query: &str) {
        let invidious = self.model().invidious();
        let instance = invidious.get_instance();
        self.imp()
            .search_instance_indicator
            .set_uri(instance.uri.clone());

        let grid = &self.imp().search_grid;
        grid.set_state(ResultPageState::Loading);
        grid.set_state(match instance.search(query).await {
            Ok(content) => {
                if content.len() == 0 {
                    ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "No Search Results".into(),
                        "Check your query for typos and simplify it if possible".into(),
                    ))
                } else {
                    grid.set_content(content);
                    ResultPageState::Success
                }
            }
            Err(error) => ResultPageState::Error(error.to_string()),
        });
    }

    async fn build_popular(&self) {
        let invidious = self.model().invidious();
        let grid = &self.imp().popular_grid;
        grid.set_state(ResultPageState::Loading);
        grid.set_state(if let Ok(instance) = invidious.get_popular_instance() {
            self.imp()
                .popular_instance_indicator
                .set_uri(instance.uri.clone());
            match instance.popular().await {
                Ok(content) => {
                    if content.len() == 0 {
                        ResultPageState::Message((
                            "dotted-box-symbolic".into(),
                            "No Popular Videos".into(),
                            "This instance supports popular videos but none exist".into(),
                        ))
                    } else {
                        grid.set_content(content);
                        ResultPageState::Success
                    }
                }
                Err(error) => ResultPageState::Error(error.to_string()),
            }
        } else {
            ResultPageState::Error("None of your instances support popular videos".into())
        });
    }

    async fn build_trending(&self) {
        let invidious = self.model().invidious();
        let grid = &self.imp().trending_grid;

        grid.set_state(ResultPageState::Loading);
        grid.set_state(if let Ok(instance) = invidious.get_trending_instance() {
            self.imp()
                .trending_instance_indicator
                .set_uri(instance.uri.clone());
            match instance.trending().await {
                Ok(content) => {
                    if content.len() == 0 {
                        ResultPageState::Message((
                            "dotted-box-symbolic".into(),
                            "No Trending Videos".into(),
                            "This instance supports trending videos but none exist".into(),
                        ))
                    } else {
                        grid.set_content(content);
                        ResultPageState::Success
                    }
                }
                Err(error) => ResultPageState::Error(error.to_string()),
            }
        } else {
            ResultPageState::Error("None of your instances support popular videos".into())
        });
    }

    fn fetch_startup(&self) {
        MainContext::default().spawn_local(clone!(@weak self as window => async move {
            futures::join!(window.build_popular(), window.build_trending());
        }));
    }
}
