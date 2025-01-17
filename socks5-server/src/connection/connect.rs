//! Socks5 command type `Connect`

use socks5_proto::{Address, Reply, Response};
use std::{
    io::Error,
    marker::PhantomData,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::TcpStream,
};

/// Socks5 command type `Connect`
///
/// By [`wait_request()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Authenticated.html#method.wait_request) on an [`Authenticated`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Authenticated.html) from SOCKS5 client, you may get a `Connect<NeedReply>`. After replying the client using [`reply()`](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Connect.html#method.reply), you will get a `Connect<Ready>`, which can be used as a regular async TCP stream.
///
/// A `Connect<S>` can be converted to a regular tokio [`TcpStream`](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html) by using the `From` trait.
#[derive(Debug)]
pub struct Connect<S> {
    stream: TcpStream,
    _state: PhantomData<S>,
}

/// Marker type indicating that the connection needs to be replied.
#[derive(Debug)]
pub struct NeedReply;

/// Marker type indicating that the connection is ready to use as a regular TCP stream.
#[derive(Debug)]
pub struct Ready;

impl Connect<NeedReply> {
    #[inline]
    pub(super) fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: PhantomData,
        }
    }

    /// Reply to the SOCKS5 client with the given reply and address.
    ///
    /// If encountered an error while writing the reply, the error alongside the original `TcpStream` is returned.
    pub async fn reply(
        mut self,
        reply: Reply,
        addr: Address,
    ) -> Result<Connect<Ready>, (Error, TcpStream)> {
        let resp = Response::new(reply, addr);

        if let Err(err) = resp.write_to(&mut self.stream).await {
            return Err((err, self.stream));
        }

        Ok(Connect::<Ready>::new(self.stream))
    }

    /// Causes the other peer to receive a read of length 0, indicating that no more data will be sent. This only closes the stream in one direction.
    #[inline]
    pub async fn shutdown(&mut self) -> Result<(), Error> {
        self.stream.shutdown().await
    }

    /// Returns the local address that this stream is bound to.
    #[inline]
    pub fn local_addr(&self) -> Result<SocketAddr, Error> {
        self.stream.local_addr()
    }

    /// Returns the remote address that this stream is connected to.
    #[inline]
    pub fn peer_addr(&self) -> Result<SocketAddr, Error> {
        self.stream.peer_addr()
    }

    /// Reads the linger duration for this socket by getting the `SO_LINGER` option.
    ///
    /// For more information about this option, see [set_linger](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Connect.html#method.set_linger).
    #[inline]
    pub fn linger(&self) -> Result<Option<Duration>, Error> {
        self.stream.linger()
    }

    /// Sets the linger duration of this socket by setting the `SO_LINGER` option.
    ///
    /// This option controls the action taken when a stream has unsent messages and the stream is closed. If `SO_LINGER` is set, the system shall block the process until it can transmit the data or until the time expires.
    ///
    /// If `SO_LINGER` is not specified, and the stream is closed, the system handles the call in a way that allows the process to continue as quickly as possible.
    #[inline]
    pub fn set_linger(&self, dur: Option<Duration>) -> Result<(), Error> {
        self.stream.set_linger(dur)
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// For more information about this option, see [set_nodelay](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Connect.html#method.set_nodelay).
    #[inline]
    pub fn nodelay(&self) -> Result<bool, Error> {
        self.stream.nodelay()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    ///
    /// If set, this option disables the Nagle algorithm. This means that segments are always sent as soon as possible, even if there is only a small amount of data. When not set, data is buffered until there is a sufficient amount to send out, thereby avoiding the frequent sending of small packets.
    pub fn set_nodelay(&self, nodelay: bool) -> Result<(), Error> {
        self.stream.set_nodelay(nodelay)
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    ///
    /// For more information about this option, see [set_ttl](https://docs.rs/socks5-server/latest/socks5_server/connection/struct.Connect.html#method.set_ttl).
    pub fn ttl(&self) -> Result<u32, Error> {
        self.stream.ttl()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    ///
    /// This value sets the time-to-live field that is used in every packet sent from this socket.
    pub fn set_ttl(&self, ttl: u32) -> Result<(), Error> {
        self.stream.set_ttl(ttl)
    }
}

impl Connect<Ready> {
    #[inline]
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            _state: PhantomData,
        }
    }
}

impl Deref for Connect<Ready> {
    type Target = TcpStream;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for Connect<Ready> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}

impl AsyncRead for Connect<Ready> {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for Connect<Ready> {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl<S> From<Connect<S>> for TcpStream {
    #[inline]
    fn from(conn: Connect<S>) -> Self {
        conn.stream
    }
}
