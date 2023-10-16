use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;

use std::cell::{Cell, RefCell};

use crate::api::Channel;
use crate::widgets::async_image::AsyncImage;

mod imp {

    use crate::utils::format_number_magnitude;

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::ChannelButton)]
    #[template(resource = "/dev/quark97/Pryvid/channel_button.ui")]
    pub struct ChannelButton {
        #[template_child]
        pub subscribers_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub image: TemplateChild<AsyncImage>,

        #[property(get, set)]
        pub title: RefCell<String>,
        #[property(get, set = Self::set_subscribers)]
        pub subscribers: Cell<u64>,
        #[property(get, set)]
        pub thumbnail: RefCell<String>,
        #[property(get, set)]
        pub author_id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannelButton {
        const NAME: &'static str = "ChannelButton";
        type Type = super::ChannelButton;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for ChannelButton {}
    impl WidgetImpl for ChannelButton {}
    impl ButtonImpl for ChannelButton {
        fn clicked(&self) {
            let obj = self.obj();
            obj.activate_action("win.open-channel", Some(&obj.author_id().to_variant()))
                .unwrap();
        }
    }

    impl ChannelButton {
        fn set_subscribers(&self, subscribers: u64) {
            self.subscribers_label.set_text(&format!(
                "{} subscribers",
                format_number_magnitude(subscribers)
            ));
            self.subscribers.set(subscribers);
        }
    }
}

glib::wrapper! {
    pub struct ChannelButton(ObjectSubclass<imp::ChannelButton>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ChannelButton {
    pub fn new(channel: &Channel) -> Self {
        let thumbnail_uri = channel.thumbnails.last().cloned().map(|x| x.uri);
        Object::builder()
            .property("title", &channel.title)
            .property("subscribers", channel.subscribers)
            .property("thumbnail", thumbnail_uri.unwrap_or_default())
            .property("author-id", &channel.id)
            .build()
    }
}
