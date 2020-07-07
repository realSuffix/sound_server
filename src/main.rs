mod connection_handler;

use crate::connection_handler::ConnectionHandler;
use std::net::TcpListener;
use std::thread::spawn;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        let stream = stream?;
        spawn(move || {
            ConnectionHandler::handle(stream);
        });
    }
    Ok(())
}
