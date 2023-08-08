use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{BindingFlags, Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::RefCell;

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
        author_label: TemplateChild<gtk::Label>,

        #[property(get, set)]
        thumbnail: RefCell<String>,
        #[property(get, set)]
        title: RefCell<String>,
        #[property(get, set)]
        author: RefCell<String>,
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
            obj.bind_property::<gtk::Label>("author", self.author_label.as_ref(), "label")
                .flags(BindingFlags::SYNC_CREATE)
                .build();
        }
    }
    impl WidgetImpl for VideoButton {}
    impl ButtonImpl for VideoButton {}
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
        Object::builder()
            .property("thumbnail", thumbnail_url)
            .property("title", &video.title)
            .property("author", &video.author)
            .build()
    }
}
