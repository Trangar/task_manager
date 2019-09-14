extern crate task_manager;

use task_manager::net::{TcpListener, TcpStream};
use task_manager::{Context, Runtime, TryTask};

fn main() {
    let listener: TcpListener = TcpListener::bind((std::net::Ipv4Addr::new(127, 0, 0, 1), 8080))
        .expect("Could not bind address");
    let mut runtime = Runtime::create().expect("Could not create runtime");

    runtime
        .register_async(&listener)
        .with(ListenerTask { listener })
        .expect("Could not spawn listener task");

    runtime.run().expect("Runtime crashed");
}

struct ListenerTask {
    listener: TcpListener,
}

impl TryTask for ListenerTask {
    type Error = std::io::Error;

    fn try_execute(self: Box<Self>, context: &mut Context) -> std::io::Result<()> {
        while let Some(stream) = self.listener.accept()? {
            println!("Accepted connection from {}", stream.addr());
            context
                .register_async(&stream)
                .with(ClientTask { stream })?;
        }

        context.register_async(&self.listener).with_boxed(self)?;

        Ok(())
    }

    fn on_error(error: std::io::Error, _context: &mut Context) {
        eprintln!("TCP listener dier: {:?}", error);
    }
}

struct ClientTask {
    stream: TcpStream,
}

impl TryTask for ClientTask {
    type Error = std::io::Error;

    fn try_execute(mut self: Box<Self>, context: &mut Context) -> std::io::Result<()> {
        let buffer = self.stream.read_until_block()?;
        println!("Client send {:?}", std::str::from_utf8(&buffer));

        context.register_async(&self.stream).with_boxed(self)?;

        Ok(())
    }

    fn on_error(error: std::io::Error, _context: &mut Context) {
        eprintln!("Client error: {:?}", error);
    }
}
