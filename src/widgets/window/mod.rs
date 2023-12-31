use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, MainContext};
use gtk::{gio, glib};
use std::cell::OnceCell;
use std::sync::Arc;

use crate::appmodel::AppModel;
use crate::widgets::{
    channel_view::ChannelView, content_grid::ContentGrid, instance_indicator::InstanceIndicator,
    playlist_view::PlaylistView, result_page::ResultPageState, video_view::VideoView,
};

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
        #[template_child]
        pub search_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub title_stack: TemplateChild<gtk::Stack>,

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
            MainContext::default().spawn_local(clone!(@weak self as window => async move {
                window.obj().build_search("").await;
            }));
        }
    }
    impl WidgetImpl for PryvidWindow {}
    impl WindowImpl for PryvidWindow {}
    impl ApplicationWindowImpl for PryvidWindow {}
    impl AdwApplicationWindowImpl for PryvidWindow {}

    #[gtk::template_callbacks]
    impl PryvidWindow {
        #[template_callback]
        fn on_search_entry_activated(&self, search_entry: gtk::SearchEntry) {
            MainContext::default().spawn_local(
                clone!(@weak self as window, @weak search_entry => async move {
                    window.obj().build_search(search_entry.text().as_str()).await;
                }),
            );
        }
        #[template_callback]
        fn on_popular_grid_refresh(&self, _: ContentGrid) {
            MainContext::default().spawn_local(clone!(@weak self as window => async move {
                window.obj().build_popular().await;
            }));
        }
        #[template_callback]
        fn on_trending_grid_refresh(&self, _: ContentGrid) {
            MainContext::default().spawn_local(clone!(@weak self as window => async move {
                window.obj().build_trending().await;
            }));
        }
        #[template_callback]
        fn on_search_button_toggled(&self, button: gtk::ToggleButton) {
            if button.is_active() {
                self.title_stack.set_visible_child_name("search");
                self.view_stack.set_visible_child_name("search");
                self.search_entry.grab_focus();
            } else {
                self.title_stack.set_visible_child_name("popular-trending");
                self.view_stack.set_visible_child_name("popular");
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
                    let nav_view = &win.imp().navigation_view;

                    if nav_view.visible_page().unwrap().tag()
                        != Some(glib::GString::from_string_unchecked(channel_id.clone()))
                    {
                        let channel_view = ChannelView::new(win.model(), channel_id);
                        nav_view.push(&channel_view);
                    }
                }
            })
            .build();

        let open_playlist_action = gio::ActionEntry::builder("open-playlist")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(move |win: &Self, _, param| {
                if let Some(param) = param {
                    let playlist_id = param.get::<String>().unwrap();
                    let nav_view = &win.imp().navigation_view;

                    if nav_view.visible_page().unwrap().tag()
                        != Some(glib::GString::from_string_unchecked(playlist_id.clone()))
                    {
                        let playlist_view = PlaylistView::new(win.model(), playlist_id);
                        nav_view.push(&playlist_view);
                    }
                }
            })
            .build();

        let open_video_action = gio::ActionEntry::builder("open-video")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(move |win: &Self, _, param| {
                if let Some(param) = param {
                    let video_id = param.get::<String>().unwrap();
                    let nav_view = &win.imp().navigation_view;

                    if nav_view.visible_page().unwrap().tag()
                        != Some(glib::GString::from_string_unchecked(video_id.clone()))
                    {
                        let video_view = VideoView::new(win.model(), video_id);
                        win.bind_property::<VideoView>(
                            "fullscreened",
                            video_view.as_ref(),
                            "fullscreened",
                        )
                        .sync_create()
                        .bidirectional()
                        .build();
                        nav_view.push(&video_view);
                    }
                }
            })
            .build();

        let fullscreen_action = gio::ActionEntry::builder("fullscreen")
            .parameter_type(None)
            .activate(move |win: &Self, _, _param| {
                win.fullscreen();
            })
            .build();
        let unfullscreen_action = gio::ActionEntry::builder("unfullscreen")
            .parameter_type(None)
            .activate(move |win: &Self, _, _param| {
                win.unfullscreen();
            })
            .build();
        let toggle_fullscreen_action = gio::ActionEntry::builder("toggle-fullscreen")
            .parameter_type(None)
            .activate(move |win: &Self, _, _param| {
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

        // Not sure if I'm happy with this or not
        // As of September 1st, it serves its purpose
        let escape_pressed_action = gio::ActionEntry::builder("escape-pressed")
            .parameter_type(None)
            .activate(move |win: &Self, _, _| {
                let visible_page = win.imp().navigation_view.visible_page().unwrap();
                if visible_page.downcast::<VideoView>().is_ok() {
                    win.unfullscreen();
                } else {
                    win.imp().search_button.set_active(false);
                    // win.imp().search_bar.set_search_mode(false);
                }
            })
            .build();

        self.add_action_entries([
            notify_action,
            open_channel_action,
            open_video_action,
            open_playlist_action,
            fullscreen_action,
            unfullscreen_action,
            toggle_fullscreen_action,
            escape_pressed_action,
        ]);
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    async fn build_search(&self, query: &str) {
        let grid = &self.imp().search_grid;
        if query.is_empty() {
            grid.set_refreshable(false);
            grid.set_content([].as_slice());
            grid.set_state(ResultPageState::Message((
                "system-search-symbolic".to_string(),
                "Search for something".to_string(),
                "If you can't find what you're looking for, simplify your search a little bit."
                    .to_string(),
            )));
            return;
        }
        grid.set_refreshable(true);

        let invidious = self.model().invidious();
        let instance = invidious.get_instance();
        self.imp()
            .search_instance_indicator
            .set_uri(instance.uri.clone());

        grid.set_state(ResultPageState::Loading);
        grid.set_state(match instance.search(query).await {
            Ok(content) => {
                grid.set_content(content.as_slice());
                if content.is_empty() {
                    ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "No Search Results".into(),
                        "Check your query for typos and simplify it if possible".into(),
                    ))
                } else {
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
                    grid.set_content(content.as_slice());
                    if content.is_empty() {
                        ResultPageState::Message((
                            "dotted-box-symbolic".into(),
                            "No Popular Videos".into(),
                            "This instance supports fetching popular videos but none exist".into(),
                        ))
                    } else {
                        ResultPageState::Success
                    }
                }
                Err(error) => ResultPageState::Error(error.to_string()),
            }
        } else {
            ResultPageState::Error("None of your instances support fetching popular videos".into())
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
                    grid.set_content(content.as_slice());
                    if content.is_empty() {
                        ResultPageState::Message((
                            "dotted-box-symbolic".into(),
                            "No Trending Videos".into(),
                            "This instance supports fetching trending videos but none exist".into(),
                        ))
                    } else {
                        ResultPageState::Success
                    }
                }
                Err(error) => ResultPageState::Error(error.to_string()),
            }
        } else {
            ResultPageState::Error("None of your instances support fetching trending videos".into())
        });
    }

    fn fetch_startup(&self) {
        MainContext::default().spawn_local(clone!(@weak self as window => async move {
            futures::join!(window.build_popular(), window.build_trending());
        }));
    }
}
