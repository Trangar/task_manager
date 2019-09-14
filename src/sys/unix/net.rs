use super::AsyncNotifier;
use std::io::Read;
use std::net::{
    SocketAddr, TcpListener as StdTcpListener, TcpStream as StdTcpStream, ToSocketAddrs,
};
use std::os::unix::io::{AsRawFd, RawFd};

pub struct TcpListener(StdTcpListener);

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let listener = StdTcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(listener.into())
    }
}

impl From<StdTcpListener> for TcpListener {
    fn from(listener: StdTcpListener) -> TcpListener {
        listener
            .set_nonblocking(true)
            .expect("Could not switch stream to non-blocking");
        TcpListener(listener)
    }
}

impl TcpListener {
    pub fn accept(&self) -> std::io::Result<Option<TcpStream>> {
        match self.0.accept() {
            Ok(s) => Ok(Some(s.into())),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl AsyncNotifier for TcpListener {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

pub struct TcpStream(StdTcpStream, SocketAddr);

impl From<(StdTcpStream, SocketAddr)> for TcpStream {
    fn from((stream, addr): (StdTcpStream, SocketAddr)) -> TcpStream {
        stream
            .set_nonblocking(true)
            .expect("Could not switch stream to non-blocking");
        TcpStream(stream, addr)
    }
}

impl TcpStream {
    pub fn addr(&self) -> SocketAddr {
        self.1
    }

    pub fn read_until_block(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        loop {
            let mut b = [0u8; 1024];
            return match self.0.read(&mut b) {
                Ok(0) => {
                    if buffer.is_empty() {
                        Err(std::io::ErrorKind::BrokenPipe.into())
                    } else {
                        Ok(buffer)
                    }
                }
                Ok(n) => {
                    buffer.extend_from_slice(&b.get(..n).unwrap());
                    continue;
                }

                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if buffer.is_empty() {
                        Err(std::io::ErrorKind::BrokenPipe.into())
                    } else {
                        Ok(buffer)
                    }
                }
                Err(e) => Err(e),
            };
        }
    }
}

impl AsyncNotifier for TcpStream {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}
