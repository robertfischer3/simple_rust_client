// Imports
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::{thread, time};

// Chat server address
const SERVER_ADDR: &str = "127.0.0.1:6000";

// Max message size
const MAX_MESSAGE_SIZE: usize = 64;

fn main() {

    // Connect to server
    let mut local_socket = TcpStream::connect(SERVER_ADDR)
        .unwrap();

    // Set non-blocking mode
    local_socket.set_nonblocking(true)
        .unwrap();

    // Message channels
    let (tx, rx) = mpsc::channel::<String>();

    // Spawn receiver thread
    thread::spawn(move || {
        loop {

            // Receive message buffer
            let mut buf = vec![0; MAX_MESSAGE_SIZE];

            // Read message from socket
            match local_socket.read_exact(&mut buf) {

                Ok(_) => {

                    // Extract message
                    let msg = String::from_utf8(buf)
                        .unwrap();

                    println!("Received: {}", msg);
                },

                // Non-fatal socket errors
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => (),

                // Break on fatal error
                Err(_) => break,
            }

            // Check for incoming messages
            match rx.try_recv() {
                Ok(msg) => {

                    // Send message
                    let result = local_socket.write_all(msg.as_bytes());

                    // Check for errors
                    if let Err(err) = result {
                        println!("Failed sending: {}", err);
                        break;
                    }

                    println!("Sent: {}", msg);
                },

                // No messages
                Err(mpsc::TryRecvError::Empty) => (),

                // Sender disconnected
                Err(mpsc::TryRecvError::Disconnected) => break,
            }

            // Small delay
            thread::sleep(time::Duration::from_millis(100));
        }
    });

    println!("Enter message (or ':quit' to exit):");

    // Input loop
    loop {
        let mut input = String::new();

        // Read input
        match io::stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(error) => {
                println!("Error reading input: {}", error);
                continue;
            },
        }

        // Trim whitespace
        let msg = input.trim().to_string();

        // Write message or quit
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }

    println!("Goodbye!");
}