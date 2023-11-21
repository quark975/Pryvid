use adw::prelude::*;
use adw::subclass::prelude::*;

use glib::{MainContext, Object, Properties};
use gtk::glib;
use gtk::{gdk_pixbuf::PixbufLoader, CompositeTemplate};
use isahc::prelude::*;
use std::cell::{Cell, RefCell};

mod imp {

    use gtk::gdk_pixbuf::InterpType;

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[properties(wrapper_type = super::AsyncImage)]
    #[template(resource = "/dev/quark97/Pryvid/async_image.ui")]
    pub struct AsyncImage {
        #[template_child]
        stack: TemplateChild<gtk::Stack>,
        #[template_child]
        spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        picture: TemplateChild<gtk::Picture>,

        #[property(get, set = Self::set_uri)]
        uri: RefCell<String>,
        #[property(get, set)]
        height: Cell<i32>,
        #[property(get, set)]
        width: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AsyncImage {
        const NAME: &'static str = "AsyncImage";
        type Type = super::AsyncImage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for AsyncImage {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn constructed(&self) {
            self.parent_constructed();

            self.spinner.set_spinning(true);
            self.spinner.start();
        }
    }
    impl WidgetImpl for AsyncImage {}
    impl BinImpl for AsyncImage {}

    impl AsyncImage {
        fn set_uri(&self, uri: String) {
            // When initializing the object it is set to nothing
            // Ignore this when it happens
            if uri.is_empty() {
                return;
            }

            MainContext::default().spawn_local(
                glib::clone!(@strong uri, @weak self as _self => async move {
                    if let Ok(mut response) = isahc::get_async(uri).await {
                        if response.status().is_success() {
                            if let Ok(image_data) = response.bytes().await {
                                let pixbuf_loader = PixbufLoader::new();
                                if pixbuf_loader.write(image_data.as_slice()).is_ok() {
                                    pixbuf_loader.close().unwrap();
                                    let width = _self.obj().width();
                                    let height = _self.obj().height();
                                    let pixbuf = pixbuf_loader.pixbuf().unwrap();
                                    let pixbuf = if width > 0 && height > 0 {
                                        if let Some(scaled_pixbuf) = pixbuf.scale_simple(width, height, InterpType::Nearest) {
                                            scaled_pixbuf
                                        } else {
                                            pixbuf
                                        }
                                    } else {
                                        pixbuf
                                    };

                                    // Set the Pixbuf on the Image widget
                                    _self.picture.set_pixbuf(Some(&pixbuf));
                                    _self.picture.set_width_request(width);
                                    _self.picture.set_height_request(height);
                                    _self.stack.set_visible_child_name("picture");
                                    _self.spinner.set_spinning(false);
                                    _self.spinner.stop();
                                    return;
                                }
                            }
                        }
                    }
                    _self.stack.set_visible_child_name("error");
                }),
            );
        }
    }
}

glib::wrapper! {
    pub struct AsyncImage(ObjectSubclass<imp::AsyncImage>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for AsyncImage {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncImage {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
