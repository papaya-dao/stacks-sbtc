use std::net::TcpListener;
use std::thread::sleep;
use std::time::Duration;

const MAX_PORT: u16 = 28443;

fn port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

pub fn find_port() -> Option<u16> {
    (18443..MAX_PORT).find(|port| {
        // Double check that the port is available as checking it once leads to race conditions
        if port_is_available(*port) {
            sleep(Duration::from_millis(100));
            port_is_available(*port)
        } else {
            false
        }
    })
}
