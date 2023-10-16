use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{subclass::Signal, Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;
use std::cell::Cell;

use crate::api::Channel;
use crate::api::Content;
use crate::api::Playlist;
use crate::api::Video;
use crate::widgets::{
    channel_button::ChannelButton,
    playlist_button::PlaylistButton,
    result_page::{ResultPage, ResultPageState},
    video_button::VideoButton,
};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapped_type = super::ContentGrid)]
    #[template(resource = "/dev/quark97/Pryvid/content_grid.ui")]
    pub struct ContentGrid {
        #[template_child]
        pub flowbox: TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub result_page: TemplateChild<ResultPage>,

        #[property(get, set)]
        pub refreshable: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContentGrid {
        const NAME: &'static str = "ContentGrid";
        type Type = super::ContentGrid;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContentGrid {
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
    impl WidgetImpl for ContentGrid {}
    impl BinImpl for ContentGrid {}

    #[gtk::template_callbacks]
    impl ContentGrid {
        #[template_callback]
        fn on_result_page_refresh(&self, _: ResultPage) {
            self.obj().emit_by_name::<()>("refresh", &[]);
        }
    }
}

glib::wrapper! {
    pub struct ContentGrid(ObjectSubclass<imp::ContentGrid>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ContentGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentGrid {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_state(&self, state: ResultPageState) {
        self.imp().result_page.set_state(state);
    }

    pub fn clear(&self) {
        let flowbox = &self.imp().flowbox;
        while let Some(child) = flowbox.child_at_index(0) {
            flowbox.remove(&child);
        }
    }

    fn add_video(&self, video: &Video) {
        self.imp().flowbox.append(&VideoButton::new(video));
    }

    fn add_channel(&self, channel: &Channel) {
        self.imp().flowbox.append(&ChannelButton::new(channel));
    }

    fn add_playlist(&self, playlist: &Playlist) {
        self.imp().flowbox.append(&PlaylistButton::new(playlist));
    }

    pub fn set_content(&self, content: &[Content]) {
        self.clear();
        for item in content {
            match item {
                Content::Video(video) => self.add_video(video),
                Content::Channel(channel) => self.add_channel(channel),
                Content::Playlist(playlist) => self.add_playlist(playlist),
            }
        }
    }
    pub fn set_videos(&self, videos: &[Video]) {
        self.clear();
        videos.into_iter().for_each(|x| self.add_video(x))
    }
    pub fn set_channels(&self, channels: &[Channel]) {
        self.clear();
        channels.into_iter().for_each(|x| self.add_channel(x))
    }
    pub fn set_playlist(&self, playlist: &[Playlist]) {
        self.clear();
        playlist.into_iter().for_each(|x| self.add_playlist(x))
    }
}
