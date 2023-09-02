use glib::{closure_local, subclass::Signal, Object};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use once_cell::sync::Lazy;

use crate::api::Content;
use crate::widgets::result_page::{ResultPage, ResultPageState};
use crate::widgets::video_button::VideoButton;

use super::channel_button::ChannelButton;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/content_grid.ui")]
    pub struct ContentGrid {
        #[template_child]
        pub flowbox: TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub result_page: TemplateChild<ResultPage>,
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

    impl ObjectImpl for ContentGrid {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            self.result_page.connect_closure(
                "refresh",
                false,
                closure_local!(@watch obj => move |_: ResultPage| {
                    obj.emit_by_name::<()>("refresh", &[]);
                }),
            );
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("refresh").build()]);
            SIGNALS.as_ref()
        }
    }
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

    pub fn set_state(&self, state: ResultPageState) {
        self.imp().result_page.set_state(state);
    }

    pub fn set_content(&self, content: Vec<Content>) {
        let flowbox = &self.imp().flowbox;
        while let Some(child) = flowbox.child_at_index(0) {
            flowbox.remove(&child);
        }
        for item in content {
            match item {
                Content::Video(video) => {
                    let video_button = VideoButton::new(&video);
                    flowbox.append(&video_button);
                }
                Content::Channel(channel) => {
                    let channel_button = ChannelButton::new(&channel);
                    flowbox.append(&channel_button);
                }
                Content::Playlist(playlist) => continue, // TODO: Implement
            }
        }
    }
}
