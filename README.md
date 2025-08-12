# 🗨️ Terminal Chat - Multi-Client TCP Chat Server in Rust

A lightweight, multi-client TCP chat system written in Rust that runs in the terminal. It supports multiple concurrent clients, broadcast messaging, simple user identity, and automatic cleanup of disconnected peers.

This project demonstrates how to build a robust, real-time networked application using Rust's standard library and concurrency features — without external async runtimes or frameworks.

---

## 🚀 Features

- 📡 **Multi-client TCP server** (listens on `0.0.0.0:8080`)
- 🧠 **Per-client threads** with message broadcasting
- 🙋‍♂️ **Named users** — set your name on connection
- 🧼 **Auto-removal of disconnected clients**
- 🔍 **Client hostname resolution**
- ⚙️ **Configurable connection backlog**
- 🚫 **Enforced client limit** (default: 10)
- 🧵 **Threaded architecture** for simplicity and reliability
- 💬 **Full-duplex messaging** between clients
- ✅ Cross-platform (Linux, macOS, Windows)

---

## 🏗️ Project Structure

```

/chat-app
├── Cargo.toml
└── src
└── bin
├── server.rs  # TCP server implementation
└── client.rs  # TCP client implementation

````

---

## 🧑‍💻 Getting Started

### 1. ✅ Requirements

- Rust (stable): [Install here](https://www.rust-lang.org/tools/install)

### 2. 🔧 Setup

Clone the repo and build:

```bash
git clone https://github.com/your-username/chat-app
cd chat-app
cargo build --release
````

---

## 🖥️ Running the Server

```bash
cargo run --bin server
```

* Listens on port `8080`
* Accepts up to 10 clients
* Displays incoming client hostnames
* Cleans up dead connections automatically

### Example Output:

```
Server listening on port 8080 (max 10 clients)...
New connection from 192.168.1.20 (host.local)
New connection from 192.168.1.21
Max clients reached. Rejecting 192.168.1.22
```

---

## 👥 Running the Client

```bash
cargo run --bin client <server_address> <port>
```

Example:

```bash
cargo run --bin client 127.0.0.1 8080
```

### Features:

* Enter your name when prompted
* Start typing and press Enter to send messages
* Displays broadcast messages from all other clients
* Gracefully handles server disconnection

### Sample Chat:

```
Please Enter Your Name: Alice
You can start sending messages now. Type and press Enter.
You: Hello everyone!
Bob: Hey Alice!
Charlie: Welcome to the chat!
```

---

## ⚙️ Configuration

The following settings can be easily modified in the code:

| Setting       | File                        | Default     | Description                              |
| ------------- | --------------------------- | ----------- | ---------------------------------------- |
| `MAX_CLIENTS` | `server.rs`                 | `10`        | Maximum number of concurrent connections |
| `PORT`        | `server.rs` and `client.rs` | `8080`      | TCP port to listen/connect               |
| `BACKLOG`     | `server.rs`                 | `10`        | OS-level connection backlog              |
| `BUFFER_SIZE` | `client.rs` and `server.rs` | `1024-4096` | Message buffer size                      |

---

## 🧪 Testing Locally

Open multiple terminals and run:

```bash
# Terminal 1
cargo run --bin server

# Terminal 2
cargo run --bin client 127.0.0.1 8080

# Terminal 3
cargo run --bin client 127.0.0.1 8080
```

Chat in real-time between the clients.

---

## 📚 How It Works

### Server

* Uses `TcpListener` to accept connections
* Spawns a thread per client
* Shares a `Vec<TcpStream>` wrapped in `Arc<Mutex<_>>`
* Reads incoming messages and relays them to all other clients

### Client

* Connects to the server via `TcpStream`
* Prompts user for a name
* Spawns a receiving thread that listens for server messages
* Main thread reads user input and sends it to the server

---

## 🛡️ Security Notes

This implementation is **for educational purposes** and does **not** include:

* Message encryption
* Authentication or access control
* Flood/spam protection

These features can be added if desired — let us know if you're interested!

---

## ✨ Future Enhancements (Optional)

* [ ] Add message timestamps
* [ ] Use colors in terminal (e.g. `crossterm`)
* [ ] Chat history logging
* [ ] Slash commands (e.g. `/name`, `/quit`)
* [ ] Async version with `tokio`
* [ ] TLS encryption (`rustls`)

---

## 🤝 Contributing

Pull requests and suggestions welcome. Feel free to fork, experiment, and extend!

---

## 🧾 License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## 💬 A Simple, Elegant Chat Experience — All in the Terminal.

Start a local server, invite friends, and chat in seconds — no browser, no bloat, no distractions.

---

