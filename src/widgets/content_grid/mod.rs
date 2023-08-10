use glib::Object;
use gtk::glib;
use gtk::subclass::prelude::*;
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
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub error_status: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub nocontent_status: TemplateChild<adw::StatusPage>,
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

pub enum ContentGridState {
    Loading,
    Success(Vec<Content>),
    Error(String),
    NoContent((String, String))
}

impl ContentGrid {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn set_state(&self, state: ContentGridState) {
        self.imp()
            .stack
            .set_visible_child_name(
                match state {
                    ContentGridState::Loading => {
                        "loading"
                    },
                    ContentGridState::Success(content) => {
                        self.set_content(content);
                        "videos"
                    },
                    ContentGridState::Error(message) => {
                        self.imp().error_status.set_description(Some(&message));
                        "error"
                    },
                    ContentGridState::NoContent((title, message)) => {
                        let nocontent_status = &self.imp().nocontent_status;
                        nocontent_status.set_title(&title);
                        nocontent_status.set_description(Some(&message));
                        "nocontent"
                    },
                }
            );
    }

    fn set_content(&self, content: Vec<Content>) {
        for item in content {
            match item {
                Content::Video(video) => {
                    let video_button = VideoButton::new(&video);
                    self.imp().flowbox.append(&video_button);
                }
                Content::Channel(channel) => continue, // TODO: Implement
                Content::Playlist(playlist) => continue, // TODO: Implement
            }
        }
    }
}
