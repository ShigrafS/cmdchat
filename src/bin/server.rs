use std::io::{Read, Write};
use std::net::{TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;

use socket2::{Socket, Domain, Type, Protocol};
use dns_lookup::lookup_addr;

const MAX_CLIENTS: usize = 10;

fn handle_client(
    mut client_stream: TcpStream,
    clients: Arc<Mutex<Vec<TcpStream>>>,
) {
    let mut buffer = [0; 1024];

    loop {
        match client_stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected.");
                break;
            }
            Ok(n) => {
                let message = &buffer[..n];

                let mut clients_guard = clients.lock().unwrap();
                // Remove dead sockets first
                clients_guard.retain(|s| s.peer_addr().is_ok());

                for other_stream in clients_guard.iter_mut() {
                    if let (Ok(a), Ok(b)) = (other_stream.peer_addr(), client_stream.peer_addr()) {
                        if a != b {
                            let _ = other_stream.write_all(message);
                        }
                    }
                }
                // clients_guard lock drops here
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }

    // Remove client from the clients list after disconnect
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.retain(|s| {
        if let (Ok(a), Ok(b)) = (s.peer_addr(), client_stream.peer_addr()) {
            a != b
        } else {
            true
        }
    });
}

fn main() -> std::io::Result<()> {
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;

    let addr: SocketAddr = "0.0.0.0:8080"
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    socket.bind(&addr.into())?;
    socket.listen(10)?; // backlog

    let listener: std::net::TcpListener = socket.into();
    println!("Server listening on port 8080 (max {} clients)...", MAX_CLIENTS);

    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    for stream_result in listener.incoming() {
        match stream_result {
            Ok(mut stream) => {  // <- make stream mutable here
                let addr = stream.peer_addr()?;

                match lookup_addr(&addr.ip()) {
                    Ok(name) => println!("New connection from {} ({})", addr, name),
                    Err(_) => println!("New connection from {}", addr),
                }

                let mut clients_guard = clients.lock().unwrap();
                if clients_guard.len() >= MAX_CLIENTS {
                    println!("Max clients reached. Rejecting {}", addr);
                    let _ = stream.write_all(b"Server full.\n"); // now works because stream is mutable
                    continue;
                }

                clients_guard.push(stream.try_clone()?);
                drop(clients_guard);

                let clients_clone = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_client(stream, clients_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }

    Ok(())
}
