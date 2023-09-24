use std::{
    mem::MaybeUninit,
    os::fd::{FromRawFd, OwnedFd},
};

pub(crate) fn open_pty() -> crate::error::Result<(OwnedFd, OwnedFd)> {
    unsafe {
        // Open the master pty
        let master_pty = libc::posix_openpt(libc::O_RDWR).as_result()?;

        // Configure the master pty : set it into raw mod so every sequences written by the user are send to the server.
        let mut settings = MaybeUninit::<libc::termios>::uninit();
        libc::tcgetattr(master_pty, settings.as_mut_ptr()).as_result()?;
        libc::cfmakeraw(settings.as_mut_ptr());
        libc::tcsetattr(master_pty, libc::TCSANOW, settings.as_mut_ptr()).as_result()?;

        // Configuration is over, allow opening of slave pty
        libc::grantpt(master_pty).as_result()?;
        libc::unlockpt(master_pty).as_result()?;

        // Get slave pty path
        let pts_name = libc::ptsname(master_pty);
        if pts_name.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }

        // Open slave pty handle
        let slave_pty = libc::open(pts_name, libc::O_RDWR).as_result()?;

        // Set slave pty to non blocking
        let flags = libc::fcntl(slave_pty, libc::F_GETFL, 0).as_result()?;
        libc::fcntl(slave_pty, libc::F_SETFL, flags | libc::O_NONBLOCK).as_result()?;

        Ok((
            OwnedFd::from_raw_fd(master_pty),
            OwnedFd::from_raw_fd(slave_pty),
        ))
    }
}

pub(crate) trait LibcResultExt: Sized {
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
