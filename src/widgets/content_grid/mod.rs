use gtk::subclass::prelude::*;
use glib::Object;
use gtk::glib;
use gtk::CompositeTemplate;

use crate::api::Content;

use super::video_button::VideoButton;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/content_grid.ui")]
    pub struct ContentGrid {
        #[template_child]
        pub flowbox: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ContentGrid {
        const NAME: &'static str = "ContentGrid";
        type Type = super::ContentGrid;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ContentGrid {}
    impl WidgetImpl for ContentGrid {}
    impl BoxImpl for ContentGrid {}
}


glib::wrapper! {
    pub struct ContentGrid(ObjectSubclass<imp::ContentGrid>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl ContentGrid {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_content(&self, content: Vec<Content>) {
        for item in content {
            match item {
                Content::Video(video) => {
                    let video_button = VideoButton::new(&video);
                    self.imp().flowbox.append(&video_button);
                },
                Content::Channel(channel) => {
                    continue
                },
                Content::Playlist(playlist) => {
                    continue
                }
            }
        }
    }
}