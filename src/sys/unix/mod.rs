#[macro_export]
macro_rules! unix_try {
    ($e:expr) => {{
        let result = $e;
        if result == -1 {
            return Err(std::io::Error::last_os_error());
        }
        result
    }};
}

pub mod net;

use crate::core::ID;
pub use epoll::Events;
use epoll::{ControlOptions, Event};
use std::num::NonZeroI32;
use std::os::unix::io::RawFd;

pub struct SysRuntime {
    epoll: RawFd,
    handles: Vec<Option<NonZeroI32>>,
}

pub trait AsyncNotifier {
    #[doc(hidden)]
    fn as_raw_fd(&self) -> RawFd;
}

impl SysRuntime {
    pub(crate) fn create() -> std::io::Result<Self> {
        let epoll = epoll::create(false)?;
        Ok(SysRuntime {
            epoll,
            handles: Vec::new(),
        })
    }

    pub(crate) fn register(&mut self, id: ID, fd: RawFd) -> std::io::Result<()> {
        epoll::ctl(
            self.epoll,
            ControlOptions::EPOLL_CTL_ADD,
            fd,
            Event::new(Events::EPOLLIN | Events::EPOLLONESHOT, id.to_u64()),
        )?;

        if id.index() >= self.handles.len() {
            self.handles.resize_with(id.index() + 1, || None);
        }
        match self.handles.get_mut(id.index()) {
            Some(h) => *h = NonZeroI32::new(fd),
            None => unreachable!(),
        }

        Ok(())
    }

    fn deregister(&mut self, id: ID) -> std::io::Result<()> {
        if let Some(fd) = self.handles.get_mut(id.index()).and_then(|o| o.take()) {
            epoll::ctl(
                self.epoll,
                ControlOptions::EPOLL_CTL_DEL,
                fd.get(),
                Event::new(Events::empty(), id.to_u64()),
            )?;
        } else {
            eprintln!(
                "Tried to remove ID {} but it is not registered, this is probably a bug",
                id
            );
        }
        Ok(())
    }

    pub(crate) fn wait(&mut self, timeout: Option<u32>) -> std::io::Result<Vec<(Events, ID)>> {
        let mut buf = vec![Event::new(Events::empty(), 0); 10];
        let new_len = epoll::wait(
            self.epoll,
            timeout.map(|t| t as i32).unwrap_or(-1),
            &mut buf,
        )?;
        let mut result = Vec::with_capacity(new_len);
        for event in buf.into_iter().take(new_len) {
            let id = unsafe { ID::from_u64(event.data) };
            self.deregister(id)?;
            result.push((
                Events::from_bits_truncate(event.events),
                // This is the value we passed in through `Context::register`, and thus is guaranteed to be a valid ID
                // If this is incorrect, and epoll can return other types of data, let us know!
                id,
            ));
        }
        Ok(result)
    }
}
