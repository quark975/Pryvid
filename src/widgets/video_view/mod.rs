use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;
use gtk::glib::{clone, MainContext};
use gtk::{gio, glib};
use gtk::{template_callbacks, CompositeTemplate};
use once_cell::sync::OnceCell;
use std::sync::Arc;

use crate::api::{DetailedVideo, Video};
use crate::appmodel::AppModel;
use crate::utils::format_number_magnitude;
use crate::widgets::async_image::AsyncImage;
use crate::widgets::instance_indicator::InstanceIndicator;
use crate::widgets::result_page::{ResultPage, ResultPageState};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/video_view.ui")]
    pub struct VideoView {
        pub video: OnceCell<DetailedVideo>,
        pub model: OnceCell<Arc<AppModel>>,

        #[template_child]
        pub instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub result_page: TemplateChild<ResultPage>,
        #[template_child(id = "video")]
        pub video_widget: TemplateChild<gtk::Video>,
        #[template_child]
        pub aspect_frame: TemplateChild<gtk::AspectFrame>,

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

    impl ObjectImpl for VideoView {}
    impl WidgetImpl for VideoView {}
    impl NavigationPageImpl for VideoView {}

    #[template_callbacks]
    impl VideoView {
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

    fn set_video(&self, video: DetailedVideo) {
        let imp = self.imp();

        // Set metadata
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

        let selected_stream = video.format_streams.last().unwrap();

        let dimensions: Vec<f32> = selected_stream
            .size
            .split("x")
            .into_iter()
            .map(|x| u32::from_str_radix(x, 10).unwrap() as f32)
            .collect();
        let ratio = dimensions[0] / dimensions[1];
        imp.aspect_frame.set_ratio(ratio);

        let file = gio::File::for_uri(&selected_stream.url);
        imp.video_widget.set_file(Some(&file));

        imp.video.set(video).unwrap();
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
