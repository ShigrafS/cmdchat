use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use colored::*; // <-- new import

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("{}", "Usage: <client> <server-address> <port>".yellow());
        return Ok(());
    }

    let addr = format!("{}:{}", args[1], args[2]);
    let resolved = addr.to_socket_addrs()?.next().expect("Unable to resolve address");

    println!("{}", format!("Connecting to {}...", resolved).yellow());
    let stream = TcpStream::connect(resolved)?;
    println!("{}", "Connected.".yellow());

    print!("{}", "Please Enter Your Name: ".blue());
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    name = name.trim().to_string();

    println!("{}", "You can start sending messages now. Type and press Enter.".yellow());

    let read_stream = stream.try_clone()?;
    let write_stream = stream;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let name_clone = name.clone();

    ctrlc::set_handler(move || {
        println!("\n{}", "Received Ctrl+C, exiting...".yellow());
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    // Reader Thread
    let running_clone = running.clone();
    thread::spawn(move || {
        let mut reader = BufReader::new(read_stream);
        let mut buffer = String::new();
        while running_clone.load(Ordering::SeqCst) {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => {
                    println!("\n{}", "Connection closed by server.".yellow());
                    std::process::exit(0);
                }
                Ok(_) => {
                    let msg = buffer.trim_end();

                    if msg.starts_with(&format!("{}:", name_clone)) {
                        // This is our own message echoed back from server
                        // Extract message content after the name prefix
                        if let Some(content) = msg.strip_prefix(&format!("{}: ", name_clone)) {
                            println!("\r{} {}", "You:".bright_green().bold(), content);
                        } else {
                            println!("\r{}", msg);
                        }
                    } else if let Some((prefix, rest)) = msg.split_once(": ") {
                        // Message from other users - prefix is their name
                        println!(
                            "\r{} {}",
                            prefix.bright_cyan().bold(),
                            rest
                        );
                    } else {
                        // If message does not match expected format, just print it raw
                        println!("\r{}", msg);
                    }

                    print!("{}", "You: ".blue());
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("\nError reading from server: {}", e);
                    break;
                }
            }
        }
    });

    // Writing Loop
    let mut input = String::new();
    let mut writer = write_stream;
    while running.load(Ordering::SeqCst) {
        input.clear();
        print!("{}", "You: ".blue());
        io::stdout().flush()?;

        if io::stdin().read_line(&mut input)? == 0 {
            println!("\n{}", "EOF detected, exiting...".yellow());
            break;
        }

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

    println!("{}", "Closing connection.".yellow());
    Ok(())
}
