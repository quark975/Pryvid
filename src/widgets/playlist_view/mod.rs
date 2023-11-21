use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, MainContext, Object, Properties};
use gtk::glib;
use gtk::CompositeTemplate;
use std::cell::{OnceCell, RefCell};
use std::sync::Arc;

use crate::appmodel::AppModel;
use crate::widgets::{
    content_grid::ContentGrid, instance_indicator::InstanceIndicator, result_page::ResultPageState,
};

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[template(resource = "/dev/quark97/Pryvid/playlist_view.ui")]
    #[properties(wrapper_type = super::PlaylistView)]
    pub struct PlaylistView {
        pub model: OnceCell<Arc<AppModel>>,

        #[property(get, set)]
        pub playlist_id: RefCell<String>,

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
            klass.bind_template_callbacks();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PlaylistView {
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
    impl WidgetImpl for PlaylistView {}
    impl NavigationPageImpl for PlaylistView {}

    #[gtk::template_callbacks]
    impl PlaylistView {
        #[template_callback]
        fn on_refresh(&self, _: ContentGrid) {
            self.obj().fetch_content();
        }
    }
}

glib::wrapper! {
    pub struct PlaylistView(ObjectSubclass<imp::PlaylistView>)
        @extends adw::NavigationPage, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl PlaylistView {
    pub fn new(model: Arc<AppModel>, playlist_id: String) -> Self {
        let obj: Self = Object::builder()
            .property("playlist-id", playlist_id)
            .build();
        obj.imp().model.set(model).unwrap();
        obj.fetch_content();
        obj
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn fetch_content(&self) {
        MainContext::default().spawn_local(clone!(@weak self as obj => async move {
            let imp = obj.imp();
            let instance = obj.model().invidious().get_instance();
            let playlist_id = obj.playlist_id();

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
                Err(error) => imp.videos_grid.set_state(ResultPageState::Error(error.to_string())),
            }
        }));
    }
}
