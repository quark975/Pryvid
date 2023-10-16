use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::{clone, closure_local, MainContext, Object};
use gtk::glib;
use gtk::{CompositeTemplate, Ordering};
use std::cell::OnceCell;
use std::sync::Arc;

use crate::api::Instances;
use crate::appmodel::AppModel;
use crate::widgets::curation_instance_row::CurationInstanceRow;

use super::curation_instance_row::PingState;

mod imp {

    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/curation_window.ui")]
    pub struct CurationWindow {
        pub model: OnceCell<Arc<AppModel>>,
        pub instances: OnceCell<Instances>,
        #[template_child]
        pub instances_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CurationWindow {
        const NAME: &'static str = "CurationWindow";
        type Type = super::CurationWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for CurationWindow {}
    impl WidgetImpl for CurationWindow {}
    impl WindowImpl for CurationWindow {}
    impl AdwWindowImpl for CurationWindow {}
}

glib::wrapper! {
    pub struct CurationWindow(ObjectSubclass<imp::CurationWindow>)
        @extends adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Native, gtk::Buildable, gtk::ConstraintTarget, gtk::Root, gtk::ShortcutManager;
}

impl CurationWindow {
    pub fn new(model: Arc<AppModel>, instances: Instances) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().model.set(model).unwrap();
        obj.imp().instances.set(instances).unwrap();
        obj.build();
        obj.ping();
        obj
    }

    fn instances(&self) -> Instances {
        self.imp().instances.get().unwrap().clone()
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn toast_message(&self, message: &str) {
        self.imp()
            .toast_overlay
            .add_toast(adw::Toast::builder().title(message).build())
    }

    fn ping(&self) {
        MainContext::default().spawn_local(clone!(@weak self as window => async move {
            let instances_listbox = window.imp().instances_listbox.clone();
            let mut index = 0;
            'outer: while let Some(child) = instances_listbox.row_at_index(index) {
                index += 1;
                let row = child.downcast::<CurationInstanceRow>().unwrap();
                let instance = row.instance();

                row.set_ping_state(PingState::Pinging);

                let mut ping = 0;
                for _ in 0..3 {
                    if let Ok(result_ping) = instance.ping(None).await {
                        ping += result_ping / 3;
                    } else {
                        row.set_ping_state(PingState::Error);
                        continue 'outer
                    }
                }

                row.set_ping_state(PingState::Success(ping));
            }

            instances_listbox.set_sort_func(move |row1, row2| {
                let row1 = row1.clone().downcast::<CurationInstanceRow>().unwrap();
                let row2 = row2.clone().downcast::<CurationInstanceRow>().unwrap();
                if let PingState::Success(ping1) = row1.ping_state() {
                    if let PingState::Success(ping2) = row2.ping_state() {
                        if ping1 < ping2 {
                            Ordering::Smaller
                        } else {
                            Ordering::Larger
                        }
                    } else {
                        Ordering::Smaller
                    }
                } else if let PingState::Success(_) = row2.ping_state() {
                    Ordering::Larger
                } else {
                    Ordering::Equal
                }
            });
        }));
    }

    fn build(&self) {
        let instances = self.instances();
        let instances_listbox = &self.imp().instances_listbox;
        for instance in instances {
            {
                let info = instance.info.read().unwrap();
                if info.has_popular.is_none() || info.has_trending.is_none() {
                    continue;
                }
            }
            let is_instance_added = self.model().invidious().is_added(&instance);
            let row = CurationInstanceRow::new(instance.clone(), is_instance_added);
            row.connect_closure("toggle", false, closure_local!(@watch self as window => move |row: CurationInstanceRow| {
                let instance = row.instance();
                if row.added() {
                    if let Err(error) = window.model().invidious().remove_instance(&instance.uri) {
                        window.toast_message(&error.to_string());
                    } else {
                        row.set_added(false);
                    }
                } else {
                    window.model().invidious().push_instance(instance).unwrap();
                    row.set_added(true);
                }
            }));
            instances_listbox.append(&row);
        }
    }
}
