use adw::{prelude::*, TabBar, TabView, ToolbarView};

use gtk::WindowControls;
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
    let content = ToolbarView::new();
    let tab_bar = TabBar::builder().autohide(false).expand_tabs(true).build();
    content.add_top_bar(&tab_bar);

    let end_control = WindowControls::new(gtk::PackType::End);
    tab_bar.set_end_action_widget(Some(&end_control));

    let tab_view = TabView::builder().hexpand(true).vexpand(true).build();
    content.set_content(Some(&tab_view));
    tab_bar.set_view(Some(&tab_view));

    for _ in 0..5{
        append_pane(
            &tab_view,
            &RemotePane::builder()
                .hexpand(true)
                .vexpand(true)
                .server_addr("oupson.fr")
                .server_port(22)
                .build(),
        );
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("flatline")
        .content(&content)
        .build();

    window.present();
}

fn append_pane(tab_view: &TabView, pane: &RemotePane) {
    let page = tab_view.append(pane);

    pane.bind_property("title", &page, "title")
        .sync_create()
        .build();
}
