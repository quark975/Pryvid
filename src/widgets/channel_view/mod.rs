use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, MainContext, Object};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::OnceCell;
use std::sync::Arc;

use crate::api::{Content, DetailedChannel, Instance};
use crate::appmodel::AppModel;
use crate::widgets::{
    channel_info_window::ChannelInfoWindow, content_grid::ContentGrid,
    instance_indicator::InstanceIndicator, result_page::ResultPageState,
};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/channel_view.ui")]
    pub struct ChannelView {
        pub model: OnceCell<Arc<AppModel>>,
        pub channel: OnceCell<DetailedChannel>,

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

    impl ObjectImpl for ChannelView {}
    impl WidgetImpl for ChannelView {}
    impl NavigationPageImpl for ChannelView {}

    #[gtk::template_callbacks]
    impl ChannelView {
        #[template_callback]
        fn on_info_button_clicked(&self, _: gtk::Button) {
            if let Some(channel) = self.channel.get() {
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
    }
}

glib::wrapper! {
    pub struct ChannelView(ObjectSubclass<imp::ChannelView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ChannelView {
    pub fn new(model: Arc<AppModel>, channel_id: String) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().model.set(model).unwrap();
        obj.set_tag(Some(&channel_id));
        obj.fetch_content(channel_id);
        obj
    }

    pub fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    async fn fetch_channel(&self, instance: &Arc<Instance>, channel_id: &str) {
        let imp = self.imp();
        let videos_grid = &imp.videos_grid;
        let channels_grid = &imp.channels_grid;

        videos_grid.set_state(ResultPageState::Loading);
        channels_grid.set_state(ResultPageState::Loading);

        match instance.channel(channel_id).await {
            Ok(channel) => {
                self.set_title(&channel.title);
                if channel.videos.is_empty() {
                    videos_grid.set_state(ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "This Channel is Empty!".into(),
                        "This channel has uploaded no videos".into(),
                    )));
                } else {
                    videos_grid.set_content(
                        channel
                            .videos
                            .clone()
                            .into_iter()
                            .map(Content::Video)
                            .collect(),
                    );
                    videos_grid.set_state(ResultPageState::Success);
                }

                if channel.related_channels.is_empty() {
                    channels_grid.set_state(ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "No Related Channels".into(),
                        "This channel has no related channels listed".into(),
                    )));
                } else {
                    channels_grid.set_content(
                        channel
                            .related_channels
                            .clone()
                            .into_iter()
                            .map(Content::Channel)
                            .collect(),
                    );
                    channels_grid.set_state(ResultPageState::Success);
                }
                imp.channel.set(channel).unwrap();
            }
            Err(error) => {
                let msg = error.to_string();
                videos_grid.set_state(ResultPageState::Error(msg.clone()));
                channels_grid.set_state(ResultPageState::Error(msg));
            }
        };
    }

    async fn fetch_playlists(&self, instance: &Arc<Instance>, channel_id: &str) {
        let playlists_grid = &self.imp().playlists_grid;

        playlists_grid.set_state(ResultPageState::Loading);

        match instance.channel_playlists(channel_id).await {
            Ok(playlists) => {
                if playlists.is_empty() {
                    playlists_grid.set_state(ResultPageState::Message((
                        "dotted-box-symbolic".into(),
                        "No Related Channels".into(),
                        "This channel has no related channels listed".into(),
                    )));
                } else {
                    playlists_grid.set_content(playlists);
                    playlists_grid.set_state(ResultPageState::Success);
                }
            }
            Err(error) => playlists_grid.set_state(ResultPageState::Error(format!("{error:?}"))),
        }
    }

    fn fetch_content(&self, channel_id: String) {
        MainContext::default().spawn_local(
            clone!(@weak self as obj, @strong channel_id => async move {
                let instance = obj.model().invidious().get_instance();

                obj.imp().instance_indicator.set_uri(instance.uri.clone());
                obj.fetch_channel(&instance, &channel_id).await;
                obj.fetch_playlists(&instance, &channel_id).await;
            }),
        );
    }
}
