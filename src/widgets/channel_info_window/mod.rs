use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::{Cell, RefCell};

use crate::api::DetailedChannel;
use crate::utils::format_number_magnitude;
use crate::widgets::async_image::AsyncImage;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[template(resource = "/dev/quark97/Pryvid/channel_info.ui")]
    #[properties(wrapper_type = super::ChannelInfoWindow)]
    pub struct ChannelInfoWindow {
        #[template_child]
        pub banner_image: TemplateChild<AsyncImage>,
        #[template_child]
        pub subscribers_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub description_label: TemplateChild<gtk::Label>,

        #[property(get, set)]
        pub banner: RefCell<String>,
        #[property(get, set)]
        pub title: RefCell<String>,
        #[property(get, set)]
        pub thumbnail: RefCell<String>,
        #[property(get, set)]
        pub description: RefCell<String>,
        #[property(get, set = Self::set_subscribers)]
        pub subscribers: Cell<u64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannelInfoWindow {
        const NAME: &'static str = "ChannelInfoWindow";
        type Type = super::ChannelInfoWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChannelInfoWindow {
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
    impl WidgetImpl for ChannelInfoWindow {}
    impl WindowImpl for ChannelInfoWindow {}
    impl AdwWindowImpl for ChannelInfoWindow {}

    impl ChannelInfoWindow {
        fn set_subscribers(&self, subscribers: u64) {
            self.subscribers_label.set_label(&format!(
                "{} subscribers",
                format_number_magnitude(subscribers)
            ));
        }
    }
}

glib::wrapper! {
    pub struct ChannelInfoWindow(ObjectSubclass<imp::ChannelInfoWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Native, gtk::Buildable, gtk::ConstraintTarget, gtk::Root, gtk::ShortcutManager;
}

impl ChannelInfoWindow {
    pub fn new(channel: &DetailedChannel) -> Self {
        Object::builder()
            .property(
                "banner",
                &channel
                    .banners
                    .iter()
                    .find(|x| x.width == 512)
                    .unwrap_or(channel.banners.last().unwrap())
                    .uri,
            )
            .property("title", &channel.title)
            .property(
                "thumbnail",
                &channel
                    .thumbnails
                    .iter()
                    .find(|x| x.width == 512)
                    .unwrap_or(channel.banners.last().unwrap())
                    .uri,
            )
            .property("description", &channel.description)
            .property("subscribers", channel.subscribers)
            .build()
    }
}
