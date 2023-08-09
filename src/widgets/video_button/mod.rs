use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{BindingFlags, Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::RefCell;
use std::time::Duration;

use crate::api::Video;
use crate::widgets::async_image::AsyncImage;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapped_type = super::VideoButton)]
    #[template(resource = "/dev/quark97/Pryvid/video_button.ui")]
    pub struct VideoButton {
        #[template_child]
        thumbnail_image: TemplateChild<AsyncImage>,
        #[template_child]
        title_label: TemplateChild<gtk::Label>,
        #[template_child]
        author_button: TemplateChild<gtk::Button>,
        #[template_child]
        length_label: TemplateChild<gtk::Label>,
        #[template_child]
        published_label: TemplateChild<gtk::Label>,
        #[template_child]
        views_label: TemplateChild<gtk::Label>,

        #[property(get, set)]
        thumbnail: RefCell<String>,
        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        author: RefCell<String>,
        #[property(get, set)]
        published: RefCell<String>,
        #[property(get, set = Self::set_views)]
        views: RefCell<u64>,
        #[property(get, set = Self::set_length)]
        length: RefCell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoButton {
        const NAME: &'static str = "VideoButton";
        type Type = super::VideoButton;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VideoButton {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();
            obj.bind_property::<AsyncImage>("thumbnail", self.thumbnail_image.as_ref(), "uri")
                .flags(BindingFlags::SYNC_CREATE)
                .build();
            obj.bind_property::<gtk::Label>("title", self.title_label.as_ref(), "label")
                .flags(BindingFlags::SYNC_CREATE)
                .build();
            obj.bind_property::<gtk::Button>("author", self.author_button.as_ref(), "label")
                .flags(BindingFlags::SYNC_CREATE)
                .build();
            obj.bind_property::<gtk::Label>("published", self.published_label.as_ref(), "label")
                .flags(BindingFlags::SYNC_CREATE)
                .build();
        }
    }
    impl WidgetImpl for VideoButton {}
    impl ButtonImpl for VideoButton {}

    impl VideoButton {
        fn set_length(&self, number: u32) {
            let string = if number == 0 {
                String::new()
            } else {
                let seconds = number % 60;
                let minutes = (number / 60) % 60;
                let hours = (number / 60) / 60;
                if hours > 0 {
                    format!("{:0>2}:{:0>2}:{:0>2}", hours, minutes, seconds)
                } else {
                    format!("{:0>2}:{:0>2}", minutes, seconds)
                }
            };
            self.length_label.set_visible(string.len() > 0);
            self.length_label.set_text(&string);
        }

        fn set_views(&self, views: u64) {
            let string = {
                if views < 1000 {
                    views.to_string()
                } else if views < 1000000 {
                    format!("{}K", views / 1000)
                } else if views < 1000000000 {
                    format!("{}M", views / 1000000)
                } else {
                    format!("{}B", views / 1000000000)
                }
            };
            self.views_label.set_text(&format!("{} views", string));
        }
    }
}

glib::wrapper! {
    pub struct VideoButton(ObjectSubclass<imp::VideoButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl VideoButton {
    pub fn new(video: &Video) -> Self {
        let thumbnail_url: &String =
            if let Some(thumbnail) = video.thumbnails.iter().find(|&x| x.quality == "medium") {
                &thumbnail.url
            } else {
                &video
                    .thumbnails
                    .first()
                    .expect("No thumbnails available")
                    .url
            };
        if video.length == 0 {
            println!("{:?}", &video);
        }
        Object::builder()
            .property("thumbnail", thumbnail_url)
            .property("title", &video.title)
            .property("author", &video.author)
            .property("length", &video.length)
            .property("published", &video.published)
            .property("views", &video.views)
            .build()
    }
}
