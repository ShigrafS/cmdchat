use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::thread;

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
    name = name.trim().to_string(); // remove newline

    println!("You can start sending messages now. Type and press Enter.");

    let stream = Arc::new(stream);

    // Clone for reading thread
    let read_stream = Arc::clone(&stream);
    thread::spawn(move || {
        let mut reader = BufReader::new(&*read_stream);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("Connection closed by server.");
                    std::process::exit(0);
                }
                Ok(_) => {
                    print!("{}", buffer);
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    });

    // Input loop (main thread)
    let mut input = String::new();
    loop {
        input.clear();
        print!("You: ");
        io::stdout().flush()?;
        if io::stdin().read_line(&mut input)? == 0 {
            break; // EOF
        }

        let message = format!("{}: {}", name, input);
        stream.write_all(message.as_bytes())?;
    }

    println!("Closing connection.");
    Ok(())
}