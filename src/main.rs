use std::{
    io,
    mem::MaybeUninit,
    os::fd::{FromRawFd, OwnedFd},
};

use adw::{prelude::*, HeaderBar};

use gtk::{Box, Orientation};
use vte4::{ApplicationExt, ApplicationExtManual, Pty, Terminal, TerminalExt};

use adw::{Application, ApplicationWindow};

mod ssh;

const APP_ID: &str = "fr.oupson.Flatline";

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt::init();
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application

    app.run()
}

fn build_ui(app: &Application) {
    let vte = Terminal::new();

    let content = Box::new(Orientation::Vertical, 0);
    // Adwaitas' ApplicationWindow does not include a HeaderBar
    content.append(&HeaderBar::new());
    content.append(&vte);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("flatline")
        .content(&content)
        .build();

    let (master_pty, slave_pty) = unsafe {
        let master_pty = libc::posix_openpt(libc::O_RDWR);
        assert!(master_pty != -1);

        let mut settings = MaybeUninit::<libc::termios>::uninit();

        libc::tcgetattr(master_pty, settings.as_mut_ptr())
            .as_result()
            .unwrap();

        libc::cfmakeraw(settings.as_mut_ptr());

        libc::tcsetattr(master_pty, libc::TCSANOW, settings.as_mut_ptr())
            .as_result()
            .unwrap();

        libc::grantpt(master_pty).as_result().unwrap();

        libc::unlockpt(master_pty).as_result().unwrap();

        let pts_name = libc::ptsname(master_pty);
        if pts_name.is_null() {
            todo!()
        }

        let slave_pty = libc::open(pts_name, libc::O_RDWR).as_result().unwrap();

        let vte_pty =
            Pty::foreign_sync(OwnedFd::from_raw_fd(master_pty), None::<&gio::Cancellable>).unwrap();

        (vte_pty, slave_pty)
    };

    vte.set_pty(Some(&master_pty));

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(ssh::ssh(slave_pty));
    });

    // Present window
    window.present();
}

trait LibcResultExt: Sized {
    fn as_result(self) -> Result<Self, io::Error>;
}

impl LibcResultExt for libc::c_int {
    fn as_result(self) -> Result<Self, io::Error> {
        if self < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(self)
        }
    }
}
