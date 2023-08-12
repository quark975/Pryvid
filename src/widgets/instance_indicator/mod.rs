use adw::subclass::prelude::*;
use adw::prelude::*;
use glib::Object;
use gtk::{glib, gio};
use gtk::glib::Properties;
use gtk::CompositeTemplate;
use std::cell::RefCell;

mod imp {

    use gtk::Align;

    use super::*;

    #[derive(Default, Debug, CompositeTemplate, Properties)]
    #[template(resource = "/dev/quark97/Pryvid/instance_indicator.ui")]
    #[properties(wrapper_type = super::InstanceIndicator)]
    pub struct InstanceIndicator {
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub instance_row: TemplateChild<adw::ActionRow>,

        #[property(get, set)]
        pub uri: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InstanceIndicator {
        const NAME: &'static str = "InstanceIndicator";
        type Type = super::InstanceIndicator;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for InstanceIndicator {
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

            self.obj().bind_property::<adw::ActionRow>("uri", self.instance_row.as_ref(), "title")
                .sync_create()
                .build();
        }
    }
    impl WidgetImpl for InstanceIndicator {}
    impl BoxImpl for InstanceIndicator {}
}


glib::wrapper! {
    pub struct InstanceIndicator(ObjectSubclass<imp::InstanceIndicator>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl InstanceIndicator {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
