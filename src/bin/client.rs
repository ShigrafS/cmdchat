use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

// Add this to Cargo.toml: ctrlc = "3"
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <server-address> <port>", args[0]);
        return Ok(());
    }

    let addr = format!("{}:{}", args[1], args[2]);
    let resolved = addr.to_socket_addrs()?.next().expect("Unable to resolve address");

    println!("Connecting to {}...", resolved);
    let stream = TcpStream::connect(resolved)?;
    println!("Connected.");

    print!("Please Enter Your Name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    name = name.trim().to_string();

    println!("You can start sending messages now. Type and press Enter.");

    // Clone the stream into two halves: one for reading, one for writing
    let read_stream = stream.try_clone()?;
    let write_stream = stream;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, exiting...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    // Reading thread
    let running_clone = running.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(read_stream);
        let mut buffer = String::new();
        while running_clone.load(Ordering::SeqCst) {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("\nConnection closed by server.");
                    std::process::exit(0);
                }
                Ok(_) => {
                    print!("{}", buffer);
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("\nError reading from server: {}", e);
                    break;
                }
            }
        }
    });

    // Writing loop (main thread)
    let mut input = String::new();
    let mut writer = write_stream;
    while running.load(Ordering::SeqCst) {
        input.clear();
        print!("You: ");
        io::stdout().flush()?;

        if io::stdin().read_line(&mut input)? == 0 {
            println!("\nEOF detected, exiting...");
            break;
        }

        // Make sure the message ends with '\n' so the server and other clients can read lines
        let message = format!("{}: {}", name, input);
        if let Err(e) = writer.write_all(message.as_bytes()) {
            eprintln!("Error sending message: {}", e);
            break;
        }
        if let Err(e) = writer.flush() {
            eprintln!("Error flushing stream: {}", e);
            break;
        }
    }

    println!("Closing connection.");
    Ok(())
}
