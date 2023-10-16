use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, MainContext, Object};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::OnceCell;
use std::sync::Arc;

use crate::appmodel::AppModel;
use crate::widgets::{
    content_grid::ContentGrid, instance_indicator::InstanceIndicator, result_page::ResultPageState,
};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/playlist_view.ui")]
    pub struct PlaylistView {
        pub model: OnceCell<Arc<AppModel>>,

        #[template_child]
        pub instance_indicator: TemplateChild<InstanceIndicator>,
        #[template_child]
        pub videos_grid: TemplateChild<ContentGrid>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlaylistView {
        const NAME: &'static str = "PlaylistView";
        type Type = super::PlaylistView;
        type ParentType = adw::NavigationPage;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistView {}
    impl WidgetImpl for PlaylistView {}
    impl NavigationPageImpl for PlaylistView {}
}

glib::wrapper! {
    pub struct PlaylistView(ObjectSubclass<imp::PlaylistView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PlaylistView {
    pub fn new(model: Arc<AppModel>, playlist_id: String) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().model.set(model).unwrap();
        obj.set_tag(Some(&playlist_id));
        obj.fetch_content(playlist_id);
        obj
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn fetch_content(&self, playlist_id: String) {
        MainContext::default().spawn_local(
            clone!(@weak self as obj, @strong playlist_id => async move {
                let imp = obj.imp();
                let instance = obj.model().invidious().get_instance();

                imp.instance_indicator.set_uri(instance.uri.clone());
                imp.videos_grid.set_state(ResultPageState::Loading);

                match instance.playlist(&playlist_id).await {
                    Ok(playlist) => {
                        // TODO: Until boundless scrolling is implemented, we need to limit to 20
                        let size = if playlist.videos.len() >= 20 {
                            20
                        } else {
                            playlist.videos.len()
                        };

                        imp.videos_grid.set_videos(&playlist.videos[..size]);
                        imp.videos_grid.set_state(ResultPageState::Success);
                        obj.set_title(&playlist.title);
                    },
                    Err(error) => imp.videos_grid.set_state(ResultPageState::Error(format!("{error:?}"))),
                }
            }),
        );
    }
}
