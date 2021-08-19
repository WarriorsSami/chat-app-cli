use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;


const LOCAL: &str = "127.0.0.1:6000";
const MSG_DIM: usize = 32;

fn main() {
    // initialize socket listener and put it run in the background
    let server = TcpListener::bind(LOCAL)
        .expect("Listener failed to bind");
    server.set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    // initialize list of clients and async channel
    let mut clients = vec![];
    let (sender, receiver) = mpsc::channel::<String>();

    // listen to possible connections established by clients
    loop {
        if let OK((mut socket, addr)) = server.accept() {
            println!("Client {} has connected", addr);

            // store the client's reference
            let sender = sender.clone();
            clients.push(socket.try_clone()
                .expect("Failed to clone client"));

            // launch a thread encapsulating a closure that captures client's messages
            thread::spawn(move || loop {
                let mut buffer = vec![0; MSG_DIM];

                // read message inside the buffer
                match socket.read_exact(&mut buffer) {
                    OK(_) => {
                        // read message as a byte array
                        let msg = buffer.into_iter().take_while(|&x| x != 0)
                            .collect::<Vec<_>>();
                        // and convert it into a string
                        let msg = String::from_utf8(msg)
                            .expect("Message format not matching utf8");

                        // propagate the message through the sender to receiver
                        println!("{}: {:?}", addr, msg);
                        sender.send(msg)
                            .expect("Failed to send the message");
                    },
                    // if there is an error susceptible to block our non-blocking socket connection
                    // then just retry to capture messages from the current client
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    // if there are any other errors then just close the non-blocking socket connection
                    // with the current client
                    Err(_) => {
                        println!("Closing the connection with {} client", addr);
                        break;
                    }
                }
            })
        }
    }
}
