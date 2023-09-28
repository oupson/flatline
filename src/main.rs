use adw::{prelude::*, TabBar, TabView, ToolbarView};

use glib::clone;
use gtk::{Button, WindowControls};
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

    let tab_bar = TabBar::builder().autohide(false).build();
    content.add_top_bar(&tab_bar);

    let end_widget = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    tab_bar.set_end_action_widget(Some(&end_widget));

    let button = Button::builder()
        .icon_name("tab-new-symbolic")
        .has_frame(false)
        .build();
    end_widget.append(&button);

    let end_control = WindowControls::new(gtk::PackType::End);
    end_widget.append(&end_control);

    let tab_view = TabView::builder().hexpand(true).vexpand(true).build();
    content.set_content(Some(&tab_view));
    tab_bar.set_view(Some(&tab_view));

    button.connect_clicked(clone!(@weak tab_view => move |_| {
        append_pane(
            &tab_view,
            &RemotePane::builder()
                .hexpand(true)
                .vexpand(true)
                .server_addr("localhost")
                .server_port(22)
                .build(),
        );
    }));

    append_pane(
        &tab_view,
        &RemotePane::builder()
            .hexpand(true)
            .vexpand(true)
            .server_addr("localhost")
            .server_port(22)
            .build(),
    );

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

    tab_view.set_selected_page(&page);
}
