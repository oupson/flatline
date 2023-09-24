use std::{
    ffi::CStr,
    os::fd::{AsRawFd, RawFd, OwnedFd},
    sync::Arc,
};

use russh_keys::{agent::client::AgentClient, key::PublicKey};
use tokio::{
    io::{unix::AsyncFd, AsyncRead, Interest},
    net::ToSocketAddrs,
};
use tracing::{error, trace};

struct Client {}

#[async_trait::async_trait]
impl russh::client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        self,
        _server_public_key: &PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
}

struct AsyncPty(AsyncFd<RawFd>);

impl AsyncRead for AsyncPty {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut guard = match self.0.poll_read_ready(cx) {
            std::task::Poll::Ready(r) => r?,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        };

        let inner_buf = unsafe { buf.unfilled_mut() };

        match guard.try_io(|g| {
            let res = unsafe {
                libc::read(
                    g.get_ref().as_raw_fd(),
                    inner_buf.as_mut_ptr() as *mut libc::c_void,
                    inner_buf.len(),
                )
            };

            if res >= 0 {
                Ok(res as usize)
            } else {
                Err(std::io::Error::last_os_error())
            }
        }) {
            Ok(res) => {
                let res = match res {
                    Ok(nbr) => {
                        unsafe { buf.assume_init(nbr) };
                        Ok(())
                    }
                    Err(e) => Err(e),
                };
                std::task::Poll::Ready(res)
            }

            Err(_would_block) => std::task::Poll::Pending,
        }
    }
}

pub async fn ssh<A>(server_addr: A, slave_pty: OwnedFd)
where
    A: ToSocketAddrs,
{
    let slave_file = tokio::io::unix::AsyncFd::new(slave_pty).unwrap();

    let mut client = AgentClient::connect_env().await.unwrap();

    let identities = client.request_identities().await.unwrap();

    let config = russh::client::Config { ..<_>::default() };
    let config = Arc::new(config);

    let sh = Client {};
    let mut session = russh::client::connect(config, server_addr, sh)
        .await
        .unwrap();

    let username = std::env::var("SSH_USERNAME").unwrap_or_else(|_| {
        unsafe { CStr::from_ptr(libc::getlogin()) }
            .to_string_lossy()
            .to_string()
    });

    trace!("username is {}", username);

    let mut is_auth = false;
    for key in identities {
        trace!("trying {}  {}", key.name(), key.fingerprint());
        let (c, r) = session.authenticate_future(&username, key, client).await;

        client = c;

        is_auth = r.unwrap();

        trace!("is auth successful : {}", is_auth);

        if is_auth {
            break;
        }
    }

    if !is_auth {
        error!("failed to authentificate");
        return;
    }

    let mut channel = session.channel_open_session().await.unwrap();

    channel
        .request_pty(true, "xterm-256color", 0, 0, 0, 0, &[])
        .await
        .unwrap();

    channel.request_shell(true).await.unwrap();

    loop {
        let mut buf1 = [0u8; 512];

        tokio::select! {
            biased;
            msg = channel.wait() => {
                if let Some(msg) = msg {
                    match msg {
                        russh::ChannelMsg::Data { ref data } => {
                            unsafe { libc::write(slave_file.as_raw_fd(), data.as_ptr() as *const libc::c_void, data.len()) };
                        }
                        _ => {}
                    }
                } else {
                    break;
                }
            }

            guard = slave_file.ready(Interest::READABLE) => {
                let mut guard = guard.unwrap();

                match guard.try_io(|g| {
                    let res = unsafe { libc::read(g.get_ref().as_raw_fd(), buf1.as_mut_ptr() as *mut libc::c_void, 512)};

                    if res >= 0 {
                        Ok(res as usize)
                    } else {
                        Err(std::io::Error::last_os_error())
                    }
                }) {
                    Ok(res) => {
                        let a = res.unwrap();
                        channel.data(&buf1[0..a]).await.unwrap();
                    }

                    Err(_would_block) => (),
                }
            }
        }
    }
}
