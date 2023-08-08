use adw::subclass::prelude::*;
use glib::Object;
use gtk::{glib, gio};
use gio::prelude::*;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::CompositeTemplate;
use glib::{subclass::InitializingObject, MainContext, Priority, Properties};
use std::cell::RefCell;

mod imp {


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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AsyncImage {
        const NAME: &'static str = "AsyncImage";
        type Type = super::AsyncImage;
        type ParentType = gtk::Box;

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
    impl BoxImpl for AsyncImage {}

    impl AsyncImage {
        fn set_uri(&self, value: String) {
            // When initializing the object it is set to nothing
            // Ignore this when it happens
            if value.len() == 0 {
                return
            }

            let main_context = MainContext::default();

            main_context.spawn_local(glib::clone!(@strong value, @weak self as _self => async move {
                let file = gio::File::for_uri(&value);
                // TODO: Do some error handling so images don't look like their loading when they failed
                match file.read_future(Priority::default()).await {
                    Ok(stream) => {
                        if let Ok(pixbuf) = Pixbuf::from_stream_future(&stream).await {
                            _self.picture.set_pixbuf(Some(&pixbuf));
                            _self.stack.set_visible_child_name("picture");
                            _self.spinner.set_spinning(false);
                            _self.spinner.stop();
                        }
                    },
                    Err(error) => {
                        println!("{:?}", error);
                    }
                }
            }));
        }
    }
}


glib::wrapper! {
    pub struct AsyncImage(ObjectSubclass<imp::AsyncImage>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl AsyncImage {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
