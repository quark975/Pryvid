use adw::prelude::*;
use adw::subclass::prelude::*;
use gio::prelude::*;
use glib::Object;
use glib::{MainContext, Priority, Properties};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::CompositeTemplate;
use gtk::{gio, glib};
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
        fn set_uri(&self, value: String) {
            // When initializing the object it is set to nothing
            // Ignore this when it happens
            if value.len() == 0 {
                return;
            }

            MainContext::default().spawn_local(
                glib::clone!(@strong value, @weak self as _self => async move {
                    let file = gio::File::for_uri(&value);
                    match file.read_future(Priority::default()).await {
                        Ok(stream) => {
                            if let Ok(pixbuf) = Pixbuf::from_stream_future(&stream).await {
                                let width = _self.obj().width();
                                let height = _self.obj().height();

                                let pixbuf = if width > 0 && height > 0 {
                                    if let Some(pixbuf) = pixbuf.scale_simple(_self.obj().width(), _self.obj().height(), InterpType::Nearest) {
                                        pixbuf
                                    } else {
                                        pixbuf
                                    }
                                } else {
                                    pixbuf
                                };

                                _self.picture.set_pixbuf(Some(&pixbuf));
                                _self.picture.set_width_request(width);
                                _self.picture.set_height_request(height);
                                _self.stack.set_visible_child_name("picture");
                                _self.spinner.set_spinning(false);
                                _self.spinner.stop();
                            } 
                        },
                        Err(error) => {
                            _self.stack.set_visible_child_name("error");
                            println!("{:?}", error);
                        }
                    }
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

impl AsyncImage {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
