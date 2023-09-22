use adw::{prelude::*, HeaderBar};

use gtk::{Box, Orientation};
use remote_pane::RemotePane;
use vte4::{ApplicationExt, ApplicationExtManual};

use adw::{Application, ApplicationWindow};

pub mod remote_pane;
mod ssh;

const APP_ID: &str = "fr.oupson.Flatline";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let content = Box::new(Orientation::Vertical, 0);
    content.append(&HeaderBar::new());

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
    content.append(&remote_pane);

    content.append(&remote_pane2);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("flatline")
        .content(&content)
        .build();

    window.present();
}

trait LibcResultExt: Sized {
    fn as_result(self) -> Result<Self, std::io::Error>;
}

impl LibcResultExt for libc::c_int {
    fn as_result(self) -> Result<Self, std::io::Error> {
        if self < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(self)
        }
    }
}
