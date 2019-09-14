use crate::net::TcpStream;
use crate::{Context, Task};
use std::io::Write;

pub struct TcpStreamWriteTask(TcpStream, Vec<u8>);

impl TcpStreamWriteTask {
    pub fn new(stream: &TcpStream, buffer: &[u8]) -> std::io::Result<Self> {
        let cloned_stream = stream.try_clone()?;
        Ok(TcpStreamWriteTask(cloned_stream, buffer.to_vec()))
    }

    pub fn run(self, context: &mut Context) {
        let task = Box::new(self);
        task.execute(context);
    }
}

impl Task for TcpStreamWriteTask {
    fn execute(mut self: Box<Self>, context: &mut Context) {
        while !self.1.is_empty() {
            let write_result = (self.0).0.write(&self.1);
            println!("Write result {:?}", write_result);
            let len = match write_result {
                Ok(len) => len,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_e) => return,
            };

            self.1.drain(..len);
        }

        if !self.1.is_empty() {
            let _ = context.register_async(&self.0).with_boxed(self);
        }
    }
}
