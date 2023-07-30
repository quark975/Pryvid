use adw::prelude::*;
use adw::subclass::prelude::*;
use adw::ResponseAppearance;
use gtk::glib::{clone, closure_local};
use gtk::{glib, CompositeTemplate};
use std::{cell::OnceCell, sync::Arc};

use crate::api::Instance;
use crate::appmodel::AppModel;
use crate::widgets::curation_window::CurationWindow;
use crate::widgets::instance_row::InstanceRow;
use crate::widgets::new_instance_window::NewInstanceWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/dev/quark97/Pryvid/preferences.ui")]
    pub struct PryvidPreferencesWindow {
        #[template_child]
        pub instances_listbox: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub popover: TemplateChild<gtk::Popover>,
        pub model: OnceCell<Arc<AppModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PryvidPreferencesWindow {
        const NAME: &'static str = "PryvidPreferencesWindow";
        type Type = super::PryvidPreferencesWindow;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PryvidPreferencesWindow {}
    impl WidgetImpl for PryvidPreferencesWindow {}
    impl WindowImpl for PryvidPreferencesWindow {}
    impl AdwWindowImpl for PryvidPreferencesWindow {}
    impl PreferencesWindowImpl for PryvidPreferencesWindow {}

    #[gtk::template_callbacks]
    impl PryvidPreferencesWindow {
        #[template_callback]
        fn on_add_button_clicked(&self, _: gtk::Button) {
            self.popover.set_visible(false);
            self.obj().show_new_instance_dialog();
        }
        #[template_callback]
        fn on_manage_button_clicked(&self, _: gtk::Button) {
            self.popover.set_visible(false);
            self.obj().show_manage_dialog();
        }
        #[template_callback]
        fn on_find_button_clicked(&self, _: gtk::Button) {
            self.popover.set_visible(false);
        }
    }
}

glib::wrapper! {
    pub struct PryvidPreferencesWindow(ObjectSubclass<imp::PryvidPreferencesWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gtk::ShortcutManager, gtk::Root, gtk::Native, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible;
}

impl PryvidPreferencesWindow {
    pub fn new(model: Arc<AppModel>) -> Self {
        let window: Self = glib::Object::builder().build();
        window.imp().model.set(model).unwrap();
        window.build();
        window
    }

    fn model(&self) -> Arc<AppModel> {
        self.imp().model.get().unwrap().clone()
    }

    fn add_instance_row(&self, instance: &Arc<Instance>) {
        let row = InstanceRow::new(
            instance.clone(),
            self.model().invidious().is_selected(instance),
        );
        row.connect_selected_notify(clone!(@weak self as window => move |row: &InstanceRow| {
            let invidious = window.model().invidious();
            invidious.select_instance(
                if row.selected() {
                    Some(row.imp().instance.get().unwrap())
                } else {
                    None
                }
            ).unwrap();
            window.rebuild();
        }));
        row.connect_closure("delete", false, closure_local!(@watch self as window => move |row: InstanceRow| {
            let dialog = adw::MessageDialog::builder()
                .heading("Remove instance?")
                .body(format!("Are you sure you want to delete '{}'?", row.title()))
                .transient_for(window)
                .build();
            dialog.add_responses(&[("cancel", "Cancel"), ("delete", "Delete")]);
            dialog.set_response_appearance("delete", ResponseAppearance::Destructive);
            dialog.connect_response(Some("delete"), clone!(@weak window, @weak row => move |_, _| {
                let instance = row.imp().instance.get().unwrap();
                window.model().invidious().remove_instance(instance.clone()).unwrap();
                window.imp().instances_listbox.remove(&row);
            }));
            dialog.present();
        }));
        self.imp().instances_listbox.append(&row);
    }

    fn rebuild(&self) {
        let listbox = &self.imp().instances_listbox;
        // TODO: If we end up using rebuild, remove_all() is a new unstable function
        loop {
            let child = listbox.row_at_index(0);
            if let Some(child) = child {
                listbox.remove(&child)
            } else {
                break;
            }
        }
        self.build()
    }

    fn build(&self) {
        let invidious = self.model().invidious();
        let instances = invidious.instances();

        // Populate with instances
        for instance in instances {
            self.add_instance_row(&instance);
        }
    }

    fn show_new_instance_dialog(&self) {
        let dialog = NewInstanceWindow::new(self.model());
        dialog.set_modal(true);
        dialog.set_transient_for(Some(self));
        dialog.connect_closure(
            "added-instance",
            false,
            closure_local!(@watch self as window => move |popup: NewInstanceWindow| {
                popup.close();
                window.rebuild();
            }),
        );
        dialog.present();
    }

    fn show_manage_dialog(&self) {
        let dialog = CurationWindow::new(self.model(), self.model().invidious().instances());
        dialog.set_modal(true);
        dialog.set_transient_for(Some(self));
        dialog.present();
    }
}
