use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
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

    // Wrap name in Arc<Mutex<>> for safe mutable access across threads
    let name_arc = Arc::new(Mutex::new(name));

    ctrlc::set_handler(move || {
        println!("\n{}", "Received Ctrl+C, exiting...".yellow());
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    // Reader Thread
    let running_clone = running.clone();
    let name_clone = Arc::clone(&name_arc);
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
                    let name_guard = name_clone.lock().unwrap();

                    if msg.starts_with(&format!("{}:", *name_guard)) {
                        // This is our own message echoed back from server
                        // Extract message content after the name prefix
                        if let Some(content) = msg.strip_prefix(&format!("{}: ", *name_guard)) {
                            println!("\n{} {}", "You:".bright_green().bold(), content);
                        } else {
                            println!("\n{}", msg);
                        }
                    } else if let Some((prefix, rest)) = msg.split_once(": ") {
                        // Message from other users - prefix is their name
                        println!("\n{} {}", prefix.bright_cyan().bold(), rest);
                    } else {
                        // If message does not match expected format, just print it raw
                        println!("\n{}", msg);
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

        let input_trimmed = input.trim();

        // Slash command support
        if input_trimmed.starts_with('/') {
            let mut parts = input_trimmed.splitn(2, ' ');
            let command = parts.next().unwrap();
            let arg = parts.next();

            match command {
                "/quit" | "/exit" => {
                    println!("{}", "Exiting chat...".yellow());
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                "/name" => {
                    if let Some(new_name) = arg {
                        let new_name = new_name.trim();
                        if new_name.is_empty() {
                            println!("{}", "Usage: /name NEWNAME".yellow());
                        } else {
                            let mut name_guard = name_arc.lock().unwrap();
                            *name_guard = new_name.to_string();
                            println!("{} {}", "Name changed to".green(), new_name.green().bold());
                        }
                    } else {
                        println!("{}", "Usage: /name NEWNAME".yellow());
                    }
                }
                _ => {
                    println!("{}", "Unknown command. Available commands: /name, /quit, /exit".yellow());
                }
            }

            continue; // don't send commands to server
        }

        // Normal message sending
        let name_guard = name_arc.lock().unwrap();
        let message = format!("{}: {}\n", *name_guard, input_trimmed);
        drop(name_guard);

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
