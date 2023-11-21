use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, MainContext, Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::{OnceCell, RefCell};
use std::sync::Arc;

use crate::api::{DetailedChannel, Instance};
use crate::appmodel::AppModel;
use crate::widgets::{
    channel_info_window::ChannelInfoWindow, content_grid::ContentGrid,
    instance_indicator::InstanceIndicator, result_page::ResultPage, result_page::ResultPageState,
};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[template(resource = "/dev/quark97/Pryvid/channel_view.ui")]
    #[properties(wrapper_type = super::ChannelView)]
    pub struct ChannelView {
        pub model: OnceCell<Arc<AppModel>>,
        pub channel: RefCell<Option<DetailedChannel>>,

        #[template_child]
        pub view_stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub videos_grid: TemplateChild<ContentGrid>,
        #[template_child]
        pub playlists_grid: TemplateChild<ContentGrid>,
        #[template_child]
        pub channels_grid: TemplateChild<ContentGrid>,
        #[template_child]
        pub instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub info_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub result_page: TemplateChild<ResultPage>,

        #[property(get, set)]
        pub channel_id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChannelView {
        const NAME: &'static str = "ChannelView";

        type Type = super::ChannelView;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChannelView {
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
    impl WidgetImpl for ChannelView {}
    impl NavigationPageImpl for ChannelView {}

    #[gtk::template_callbacks]
    impl ChannelView {
        #[template_callback]
        fn on_info_button_clicked(&self, _: gtk::Button) {
            if let Some(channel) = self.channel.borrow().as_ref() {
                let window = self
                    .obj()
                    .root()
                    .unwrap()
                    .downcast::<gtk::Window>()
                    .unwrap();
                let dialog = ChannelInfoWindow::new(channel);
                dialog.set_modal(true);
                dialog.set_transient_for(Some(&window));
                dialog.present();
            }
        }
        #[template_callback]
        fn on_refresh_clicked(&self, _: ResultPage) {
            self.obj().fetch_content();
        }
    }
}

glib::wrapper! {
    pub struct ChannelView(ObjectSubclass<imp::ChannelView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ChannelView {
    pub fn new(model: Arc<AppModel>, channel_id: String) -> Self {
        let obj: Self = Object::builder().property("channel-id", channel_id).build();
        obj.imp().model.set(model).unwrap();
        obj.fetch_content();
        obj
    }

    pub fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    async fn fetch_channel(&self, instance: &Arc<Instance>, channel_id: &str) -> ResultPageState {
        let imp = self.imp();
        let videos_grid = &imp.videos_grid;
        let channels_grid = &imp.channels_grid;

        videos_grid.set_state(ResultPageState::Loading);
        channels_grid.set_state(ResultPageState::Loading);

        match instance.channel(channel_id).await {
            Ok(channel) => {
                self.set_title(&channel.title);
                imp.channel.replace(Some(channel.clone()));
                videos_grid.set_state(if channel.videos.is_empty() {
                    ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "This Channel is Empty!".into(),
                        "This channel has uploaded no videos".into(),
                    ))
                } else {
                    videos_grid.set_videos(channel.videos.as_slice());
                    ResultPageState::Success
                });
                channels_grid.set_state(if channel.related_channels.is_empty() {
                    ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "No Related Channels".into(),
                        "This channel has no related channels listed".into(),
                    ))
                } else {
                    channels_grid.set_channels(channel.related_channels.as_slice());
                    ResultPageState::Success
                });
                ResultPageState::Success
            }
            Err(err) => ResultPageState::Error(err.to_string()),
        }
    }

    async fn fetch_playlists(&self, instance: &Arc<Instance>, channel_id: &str) -> ResultPageState {
        let playlists_grid = &self.imp().playlists_grid;

        match instance.channel_playlists(channel_id).await {
            Ok(playlists) => {
                playlists_grid.set_state(if playlists.is_empty() {
                    ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "No Related Channels".into(),
                        "This channel has no related channels listed".into(),
                    ))
                } else {
                    playlists_grid.set_playlist(playlists.as_slice());
                    ResultPageState::Success
                });
                ResultPageState::Success
            }
            Err(error) => ResultPageState::Error(error.to_string()),
        }
    }

    fn fetch_content(&self) {
        self.imp().result_page.set_state(ResultPageState::Loading);
        MainContext::default().spawn_local(clone!(@weak self as obj => async move {
            let channel_id = obj.channel_id();
            let instance = obj.model().invidious().get_instance();
            obj.imp().instance_indicator.set_uri(instance.uri.clone());

            let channel_state = obj.fetch_channel(&instance, &channel_id).await;
            let playlist_state = obj.fetch_playlists(&instance, &channel_id).await;
            obj.imp().result_page.set_state(
                if let ResultPageState::Error(_) = channel_state {
                    channel_state
                } else {
                    playlist_state
                }
            );
        }));
    }
}
