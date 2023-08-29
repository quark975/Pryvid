use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;
use gtk::glib::{clone, MainContext, Properties};
use gtk::{gio, glib};
use gtk::{template_callbacks, CompositeTemplate};
use once_cell::sync::OnceCell;
use std::cell::Cell;
use std::sync::Arc;

use crate::api::{Content, DetailedVideo, Video};
use crate::appmodel::AppModel;
use crate::utils::format_number_magnitude;
use crate::widgets::async_image::AsyncImage;
use crate::widgets::content_grid::ContentGrid;
use crate::widgets::instance_indicator::InstanceIndicator;
use crate::widgets::result_page::{ResultPage, ResultPageState};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::VideoView)]
    #[template(resource = "/dev/quark97/Pryvid/video_view.ui")]
    pub struct VideoView {
        pub video: OnceCell<DetailedVideo>,
        pub model: OnceCell<Arc<AppModel>>,

        #[property(get, set)]
        pub show_sidebar: Cell<bool>,
        #[property(get, set)]
        pub sidebar_collapsed: Cell<bool>,
        #[property(get, set = Self::set_fullscreened)]
        pub fullscreened: Cell<bool>,
        #[property(get, set)]
        pub timestamp: Cell<i64>,

        #[template_child]
        pub instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub result_page: TemplateChild<ResultPage>,
        #[template_child(id = "normal_video")]
        pub normal_video_widget: TemplateChild<gtk::Video>,
        #[template_child(id = "fullscreen_video")]
        pub fullscreen_video_widget: TemplateChild<gtk::Video>,
        #[template_child]
        pub split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub fullscreen_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub hover_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub headerbar_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        pub author_thumbnail: TemplateChild<AsyncImage>,
        #[template_child]
        pub author_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub author_subs: TemplateChild<gtk::Label>,
        #[template_child]
        pub likes_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub dislikes_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub views_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub published_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub recommended_grid: TemplateChild<ContentGrid>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoView {
        const NAME: &'static str = "VideoView";
        type Type = super::VideoView;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VideoView {
        fn constructed(&self) {
            self.parent_constructed();

            let controller = gtk::EventControllerMotion::new();
            controller.connect_enter(clone!(@weak self as imp => move |_, _, _| {
                imp.headerbar_revealer.set_reveal_child(true);
            }));
            controller.connect_leave(clone!(@weak self as imp => move |_| {
                imp.headerbar_revealer.set_reveal_child(false);
            }));
            self.hover_box.add_controller(controller);
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
    impl WidgetImpl for VideoView {}
    impl NavigationPageImpl for VideoView {
        fn hiding(&self) {
            let obj = self.obj();

            if obj.sidebar_collapsed() {
                obj.set_show_sidebar(false);
            }
            obj.set_playing(false);
            obj.set_fullscreened(false);
        }
        fn hidden(&self) {
            self.obj().destroy_media_stream();
        }
        fn showing(&self) {
            self.obj().init_media_stream();
        }
        fn shown(&self) {
            self.obj().set_playing(true);
        }
    }

    #[template_callbacks]
    impl VideoView {
        fn set_fullscreened(&self, fullscreened: bool) {
            self.fullscreened.set(fullscreened);
            self.fullscreen_stack
                .set_visible_child_name(if fullscreened { "fullscreen" } else { "normal" });
        }
        #[template_callback]
        fn on_channel_clicked(&self) {
            self.obj()
                .activate_action(
                    "win.open-channel",
                    Some(&self.video.get().unwrap().author_id.to_variant()),
                )
                .unwrap();
        }
    }
}

glib::wrapper! {
    pub struct VideoView(ObjectSubclass<imp::VideoView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl VideoView {
    pub fn new(model: Arc<AppModel>, video_id: String) -> Self {
        let obj: VideoView = Object::builder().build();
        obj.imp().model.set(model).unwrap();
        obj.fetch_video(video_id);
        obj
    }

    pub fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    pub fn set_playing(&self, playing: bool) {
        if let Some(stream) = self.imp().normal_video_widget.media_stream() {
            stream.set_playing(playing);
        }
    }

    pub fn playing(&self) -> bool {
        if let Some(stream) = self.imp().normal_video_widget.media_stream() {
            stream.is_playing()
        } else {
            false
        }
    }

    pub fn init_media_stream(&self) {
        let imp = self.imp();
        if let Some(video) = imp.video.get() {
            let url = &video.format_streams.last().unwrap().url;
            let file = gio::File::for_uri(url);
            let stream = gtk::MediaFile::for_file(&file);
            imp.normal_video_widget.set_media_stream(Some(&stream));
            imp.fullscreen_video_widget.set_media_stream(Some(&stream));
            stream.connect_prepared_notify(clone!(@weak self as obj => move |stream| {
                stream.seek(obj.timestamp());
                stream
                    .bind_property::<Self>("timestamp", obj.as_ref(), "timestamp")
                    .sync_create()
                    .build();
                stream.play();
            }));
        }
    }

    pub fn destroy_media_stream(&self) {
        let imp = self.imp();
        imp.normal_video_widget
            .set_media_stream(None::<&gtk::MediaStream>);
        imp.fullscreen_video_widget
            .set_media_stream(None::<&gtk::MediaStream>);
    }

    fn set_video(&self, video: DetailedVideo) {
        let imp = self.imp();

        /// Setup player
        let selected_stream = video.format_streams.last().unwrap();
        imp.fullscreen_stack
            .set_visible_child_name(if self.fullscreened() {
                "fullscreened"
            } else {
                "normal"
            });

        imp.video.set(video.clone()).unwrap();
        self.init_media_stream();

        // When using GtkVideo:autoplay and resizing the window, the video will play after being
        // paused. This will give the desired behavior that GtkVideo:autoplay does not
        /// Setup metadata
        self.set_title(&video.title);
        imp.author_name.set_label(&video.author);
        imp.author_subs.set_label(&video.subscribers);
        // TODO: Select a more reasonable thumbnail
        imp.author_thumbnail
            .set_uri(video.author_thumbnails.last().unwrap().url.clone());
        imp.likes_label
            .set_label(&format_number_magnitude(video.likes as u64));
        imp.dislikes_label
            .set_label(&format_number_magnitude(video.dislikes as u64));
        imp.views_label
            .set_label(&format_number_magnitude(video.views));
        imp.published_label.set_label(&video.published);
        imp.description_label.set_label(&video.description);
        imp.recommended_grid.set_content(
            video
                .recommended
                .clone()
                .into_iter()
                .map(|x| Content::Video(x))
                .collect(),
        );
        imp.recommended_grid.set_state(ResultPageState::Success);
    }

    fn fetch_video(&self, video_id: String) {
        MainContext::default().spawn_local(
            clone!(@weak self as obj, @strong video_id => async move {
                let invidious = obj.model().invidious();
                let instance = invidious.get_instance();
                let imp = obj.imp();

                imp.instance_indicator.set_uri(instance.uri.clone());
                imp.result_page.set_state(ResultPageState::Loading);

                let result = instance.video(&video_id).await;

                imp.result_page.set_state(
                    match result {
                        Ok(video) => {
                            obj.set_video(video);
                            ResultPageState::Success
                        },
                        Err(error) => ResultPageState::Error(error.to_string())
                    }
                );
            }),
        );
    }
}
