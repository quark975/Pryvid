use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::RefCell;

use crate::api::Playlist;
use crate::widgets::async_image::AsyncImage;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::PlaylistButton)]
    #[template(resource = "/dev/quark97/Pryvid/playlist_button.ui")]
    pub struct PlaylistButton {
        #[template_child]
        thumbnail_image: TemplateChild<AsyncImage>,
        #[template_child]
        title_label: TemplateChild<gtk::Label>,
        #[template_child]
        author_button: TemplateChild<gtk::Button>,
        #[template_child]
        video_count_label: TemplateChild<gtk::Label>,

        #[property(get, set)]
        thumbnail: RefCell<String>,
        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        author: RefCell<String>,
        #[property(get, set = Self::set_video_count)]
        video_count: RefCell<u64>,
        #[property(get, set)]
        author_id: RefCell<String>,
        #[property(get, set)]
        playlist_id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistButton {
        const NAME: &'static str = "PlaylistButton";
        type Type = super::PlaylistButton;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistButton {
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
    impl WidgetImpl for PlaylistButton {}
    impl BinImpl for PlaylistButton {}

    #[gtk::template_callbacks]
    impl PlaylistButton {
        fn set_video_count(&self, video_count: u64) {
            self.video_count_label
                .set_text(&format!("{video_count} videos"));
        }

        #[template_callback]
        fn on_playlist_clicked(&self, _: gtk::Button) {
            self.obj()
                .activate_action(
                    "win.open-playlist",
                    Some(&self.playlist_id.borrow().to_variant()),
                )
                .unwrap();
        }

        #[template_callback]
        fn on_author_clicked(&self, _: gtk::Button) {
            self.obj()
                .activate_action(
                    "win.open-channel",
                    Some(&self.author_id.borrow().to_variant()),
                )
                .unwrap();
        }
    }
}

glib::wrapper! {
    pub struct PlaylistButton(ObjectSubclass<imp::PlaylistButton>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PlaylistButton {
    pub fn new(playlist: &Playlist) -> Self {
        Object::builder()
            .property("thumbnail", &playlist.thumbnail)
            .property("title", &playlist.title)
            .property("author", &playlist.author)
            .property("video-count", &playlist.video_count)
            .property("author-id", &playlist.author_id)
            .property("playlist-id", &playlist.id)
            .build()
    }
}
