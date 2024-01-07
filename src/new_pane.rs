use adw::subclass::prelude::*;
use gtk::{gio, glib, prelude::*, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(file = "gtk/new_pane.ui")]
    pub struct NewPane {
        #[template_child]
        entry_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        close_pane: TemplateChild<gtk::Button>,
        #[template_child]
        new_pane: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewPane {
        const NAME: &'static str = "FlatLineNewPane";
        type Type = super::NewPane;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);

            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_accessible_role(gtk::AccessibleRole::Group);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NewPane {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    impl WidgetImpl for NewPane {}
}

glib::wrapper! {
    pub struct NewPane(ObjectSubclass<imp::NewPane>)
        @extends gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl Default for NewPane {
    fn default() -> Self {
        glib::Object::new()
    }
}

impl NewPane {
    pub fn new() -> Self {
        Self::default()
    }
}
