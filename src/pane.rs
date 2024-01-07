use std::cell::RefCell;

use adw::subclass::prelude::*;
use glib::clone;
use gtk::{gio, glib, prelude::*};

mod imp {
    use gio::{ActionEntry, SimpleActionGroup};
    use tracing::warn;

    use crate::{new_pane, remote_pane::RemotePane};

    use super::*;
    #[derive(Debug, glib::Properties)]
    #[properties(wrapper_type = super::Pane)]
    pub struct Pane {
        #[property(get, set)]
        title: RefCell<String>,
    }

    impl Default for Pane {
        fn default() -> Self {
            Self {
                title: RefCell::new("New Pane".to_owned()),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Pane {
        const NAME: &'static str = "FlatLinePane";
        type Type = super::Pane;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for Pane {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
            let self_obj: &super::Pane = &*self.obj();

            let action_new_entry = ActionEntry::builder("new-entry")
                .activate(clone!(@weak self_obj => move |_, _, _| {
                    warn!("TODO");

                    while let Some(child) = self_obj.first_child() {
                        child.unparent();
                    }

                    let remote_pane = RemotePane::builder()
                        .hexpand(true)
                        .vexpand(true)
                        .server_addr("localhost")
                        .server_port(22)
                        .build();

                    remote_pane.bind_property("title", &self_obj, "title")
                        .sync_create()
                        .build();

                    remote_pane.set_parent(&self_obj);
                }))
                .build();

            let actions = SimpleActionGroup::new();
            actions.add_action_entries([action_new_entry]);
            self_obj.insert_action_group("pane", Some(&actions));

            let new_pane = new_pane::NewPane::new();
            new_pane.set_parent(self_obj);
        }
    }

    impl WidgetImpl for Pane {}
}

glib::wrapper! {
    pub struct Pane(ObjectSubclass<imp::Pane>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for Pane {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl Pane {
    pub fn new() -> Self {
        Self::default()
    }
}
