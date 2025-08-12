use std::io::{Read, Write};
use std::net::{TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use socket2::{Socket, Domain, Type, Protocol};
use std::net::SocketAddr;
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

                // Broadcast to other clients
                let mut clients_guard = clients.lock().unwrap();
                clients_guard.retain(|s| s.peer_addr().is_ok()); // Remove dead sockets
                for other_stream in clients_guard.iter_mut() {
                    if other_stream.peer_addr() != client_stream.peer_addr() {
                        let _ = other_stream.write_all(message);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }

    // Remove the client after disconnect
    let mut clients_guard = clients.lock().unwrap();
    clients_guard.retain(|s| s.peer_addr() != client_stream.peer_addr());
}

fn main() -> std::io::Result<()> {
    // 🧱 Create listener socket with custom backlog using socket2
    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&"0.0.0.0:8080".parse::<SocketAddr>()?.into())?;
    socket.listen(10)?; // 🧠 Set custom backlog here

    let listener = socket.into_tcp_listener();
    println!("Server listening on port 8080 (max 10 clients)...");

    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr()?;
                
                // 🌐 Try to resolve client hostname
                match lookup_addr(&addr.ip()) {
                    Ok(name) => println!("New connection from {} ({})", addr, name),
                    Err(_) => println!("New connection from {}", addr),
                }

                let mut clients_guard = clients.lock().unwrap();
                if clients_guard.len() >= MAX_CLIENTS {
                    println!("Max clients reached. Rejecting {}", addr);
                    let _ = stream.write_all(b"Server full.\n");
                    continue;
                }

                clients_guard.push(stream.try_clone()?);
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