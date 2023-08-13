use adw::subclass::prelude::*;
use glib::Object;
use gtk::glib;
use gtk::glib::{clone, MainContext};
use gtk::CompositeTemplate;
use once_cell::sync::OnceCell;
use std::sync::Arc;

use crate::api::Video;
use crate::appmodel::AppModel;
use crate::widgets::instance_indicator::InstanceIndicator;
use crate::widgets::result_page::{ResultPage, ResultPageState};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/video_view.ui")]
    pub struct VideoView {
        pub video: OnceCell<Video>,
        pub model: OnceCell<Arc<AppModel>>,

        #[template_child]
        pub instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub result_page: TemplateChild<ResultPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VideoView {
        const NAME: &'static str = "VideoView";
        type Type = super::VideoView;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for VideoView {}
    impl WidgetImpl for VideoView {}
    impl NavigationPageImpl for VideoView {}
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
                            println!("{video:?}");
                            ResultPageState::Success
                        },
                        Err(error) => ResultPageState::Error(error.to_string())
                    }
                );
            }),
        );
    }
}
