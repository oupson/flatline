use std::{
    cell::{OnceCell, RefCell},
    thread::JoinHandle,
};

use anyhow::{Context, Ok};
use glib::{
    subclass::{
        prelude::{DerivedObjectProperties, ObjectImpl, ObjectImplExt},
        types::{ObjectSubclass, ObjectSubclassExt},
    },
    ObjectExt,
};
use gtk::{
    subclass::widget::{WidgetClassExt, WidgetImpl},
    Box,
};
use tokio::sync::mpsc::{self, Sender};
use tracing::{error, warn};
use vte4::{BoxExt, Pty, Terminal, TerminalExt, WidgetExt};

pub enum RemotePaneMsg {
    Close,
    SizeChanged(i32, i32),
}

#[derive(glib::Properties)]
#[properties(wrapper_type = super::RemotePane)]
pub struct RemotePane {
    #[property(get, construct_only, builder())]
    server_addr: OnceCell<String>,

    #[property(get, construct_only, minimum = 0, maximum = 65_535, builder())]
    server_port: OnceCell<u32>,

    term: Terminal,

    #[property(get, set)]
    title: RefCell<String>,

    thread_handle: RefCell<Option<JoinHandle<()>>>,

    size: RefCell<(i32, i32)>,

    sender: OnceCell<Sender<RemotePaneMsg>>,
}

impl Default for RemotePane {
    fn default() -> Self {
        let term = Terminal::builder()
            .hexpand(true)
            .vexpand(true)
            .enable_sixel(true)
            .build();

        Self {
            term,
            server_addr: OnceCell::new(),
            server_port: OnceCell::new(),
            title: RefCell::new(String::from("Not Connected")),
            thread_handle: RefCell::new(None),
            size: RefCell::new((-1, -1)),
            sender: OnceCell::new(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RemotePane {
    const NAME: &'static str = "FlatLineRemotePane";
    type Type = super::RemotePane;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BinLayout>();
    }
}

#[glib::derived_properties]
impl ObjectImpl for RemotePane {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = &*self.obj();
        let content = Box::builder().hexpand(true).vexpand(true).build();
        content.append(&self.term);
        content.set_parent(obj);

        self.term
            .bind_property("window-title", obj, "title")
            .transform_to(|bindings, term_title: String| {
                let server_addr = bindings
                    .target()
                    .map(|s| s.property::<String>("server_addr"));

                if term_title.is_empty() {
                    server_addr
                } else {
                    server_addr.map(|s| format!("{} - {}", term_title, s))
                }
            })
            .sync_create()
            .build();

        let (sender, receiver) = mpsc::channel(10);
        {
            let term_size = self.size.clone();
            let sender = sender.clone();
            self.term.connect_contents_changed(move |term| {
                let mut term_size = term_size.borrow_mut();
                if let Some(new_size) = term.pty().and_then(|pty| pty.size().ok()) {
                    if new_size != *term_size {
                        *term_size = new_size;
                        if let Err(e) =
                            sender.try_send(RemotePaneMsg::SizeChanged(new_size.1, new_size.0))
                        {
                            warn!("failed to send size data to remote : {}", e);
                        }
                    }
                }
            });
        }

        self.sender.set(sender).unwrap();

        if let Err(e) = self.spawn_ssh_session(receiver) {
            error!("failed to spawn ssh session : {}", e);
        }
    }

    fn dispose(&self) {
        if let Some(sender) = self.sender.get() {
            if let Err(e) = sender.blocking_send(RemotePaneMsg::Close) {
                warn!("failed to send close event : {}", e);
            }

            if let Some(handle) = self.thread_handle.take() {
                handle.join().unwrap();
            }
        }

        while let Some(child) = self.obj().first_child() {
            child.unparent();
        }
    }
}

impl WidgetImpl for RemotePane {}

impl RemotePane {
    fn spawn_ssh_session(&self, receiver: mpsc::Receiver<RemotePaneMsg>) -> anyhow::Result<()> {
        let addr = self
            .server_addr
            .get()
            .ok_or(anyhow::Error::msg("missing server-addr"))?;

        let port = self
            .server_port
            .get()
            .ok_or(anyhow::Error::msg("missing server-port"))?;

        let addr_with_port = format!("{}:{}", addr, port);

        let (master_pty, slave_pty) = crate::util::open_pty().context("Failed to open pty")?;
        let vte_pty = Pty::foreign_sync(master_pty, None::<&gio::Cancellable>)?;
        self.term.set_pty(Some(&vte_pty));

        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(crate::ssh::ssh(addr_with_port, slave_pty, receiver));
        });
        self.thread_handle.set(Some(handle));

        Ok(())
    }
}
