use adw::subclass::prelude::*;
use adw::prelude::*;
use glib::Object;
use gtk::Ordering;
use gtk::glib;
use gtk::CompositeTemplate;
use gtk::glib::MainContext;
use gtk::glib::Priority;
use glib::{clone, closure_local};
use std::cell::OnceCell;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use isahc::prelude::*;

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

    fn ping(&self) {
        let (sender, receiver) = MainContext::channel(Priority::default());
        let instances_listbox = self.imp().instances_listbox.clone();
        let mut uris = Vec::new();
        loop {
            if let Some(child) = instances_listbox.row_at_index(uris.len() as i32) {
                uris.push(child.downcast::<CurationInstanceRow>().unwrap().instance().uri.clone());
            } else {
                break;
            }
        }

        thread::spawn(move || {
            for (index, uri) in uris.iter().enumerate() {
                sender.send(Some((index, PingState::Pinging))).unwrap();
                let mut ping = 0;
                for _ in 0..3 {
                    let request = isahc::Request::get(uri)
                        .timeout(Duration::from_secs(5))
                        .body(()).unwrap();
                    let elapsed = Instant::now();
                    let response = request.send();
                    let elapsed = elapsed.elapsed();
                    match response {
                        Ok(_) => ping += elapsed.as_millis() / 3,
                        Err(_) => continue
                    }
                }
                sender.send(Some((index, if ping == 0 {
                    PingState::Error
                } else {
                    PingState::Success(ping)
                }))).unwrap();
            }
            sender.send(None).unwrap();
        });
        receiver.attach(None, clone!(@weak instances_listbox => @default-return Continue(false),
            move |result: Option<(usize, PingState)>| {
                match result {
                    Some((index, state)) => {
                        instances_listbox
                            .row_at_index(index as i32)
                            .and_downcast::<CurationInstanceRow>()
                            .unwrap()
                            .set_state(state);
                        Continue(true)
                    },
                    None => {
                        // Sort instances
                        instances_listbox.set_sort_func(move |row1, row2| {
                            let row1 = row1.clone().downcast::<CurationInstanceRow>().unwrap();
                            let row2 = row2.clone().downcast::<CurationInstanceRow>().unwrap();
                            if let PingState::Success(ping1) = row1.ping_state() {
                                if let PingState::Success(ping2) = row2.ping_state() {
                                    if ping1 < ping2 {
                                        return Ordering::Smaller
                                    } else {
                                        return Ordering::Larger
                                    }
                                }
                            }
                            Ordering::Larger
                        });

                        // Add "add buttons"
                        let mut index = 0;
                        loop {
                            if let Some(child) = instances_listbox.row_at_index(index) {
                                child.downcast::<CurationInstanceRow>().unwrap().set_add_button_visible(true)
                            } else {
                                break;
                            }
                            index += 1
                        }
                        Continue(false)
                    }
                }

            })
        );
    }

    fn build(&self) {
        let instances = self.instances();
        let instances_listbox = &self.imp().instances_listbox;
        for instance in instances {
            let is_instance_added = self.model().invidious().is_added(&instance);
            let row = CurationInstanceRow::new(instance.clone(), is_instance_added);
            row.connect_closure("toggle", false, closure_local!(@watch self as window => move |row: CurationInstanceRow| {
                let instance = row.instance();
                if row.added() {
                    window.model().invidious().push_instance(instance).unwrap();
                } else {
                    window.model().invidious().remove_instance(instance).unwrap();
                }
            }));
            instances_listbox.append(&row);
        }
    }
}
