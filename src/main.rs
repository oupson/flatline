use adw::{prelude::*, HeaderBar, TabBar, TabView};

use gtk::{Box, Orientation};
use remote_pane::RemotePane;
use vte4::{ApplicationExt, ApplicationExtManual};

use adw::{Application, ApplicationWindow};

pub(crate) mod error;
pub mod remote_pane;
mod ssh;
pub(crate) mod util;

const APP_ID: &str = "fr.oupson.Flatline";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let content = Box::new(Orientation::Vertical, 0);
    let header_bar = HeaderBar::new();
    content.append(&header_bar);

    let tab_bar = TabBar::builder().build();

    content.append(&tab_bar);

    let tab_view = TabView::builder().hexpand(true).vexpand(true).build();
    content.append(&tab_view);
    tab_bar.set_view(Some(&tab_view));

    let remote_pane = RemotePane::builder()
        .hexpand(true)
        .vexpand(true)
        .server_addr("oupson.fr")
        .server_port(22)
        .build();

    let remote_pane2 = RemotePane::builder()
        .hexpand(true)
        .vexpand(true)
        .server_addr("oupson.fr")
        .server_port(22)
        .build();
    tab_view.append(&remote_pane);
    tab_view.append(&remote_pane2);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("flatline")
        .content(&content)
        .build();

    window.present();
}
