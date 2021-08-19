use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::borrow::Borrow;


const LOCAL: &str = "127.0.0.1:6000";
const MSG_DIM: usize = 32;

// pause the current thread
fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn main() {
    // Initialize socket stream and put it run in the background
    let mut client = TcpStream::connect(LOCAL)
        .expect("Stream failed to connect");
    client.set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    // set an async channel for our process
    let (_sender, receiver) = mpsc::channel::<String>();

    // initialize a channel to send our client messages
    thread::spawn(move || loop {
        let mut buffer = vec![0; MSG_DIM];

        // read the client message inside our buffer
        match client.read_exact(&mut buffer) {
            Ok(_) => {
                let msg = buffer.into_iter().take_while(|x| x != 0)
                    .collect::<Vec<_>>();
                println!("The received message is {:?}", msg);
            },
            // if there is an error susceptible to block our non-blocking socket connection
            // then just retry to capture messages from the current client
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            // if there are any other errors then just close the non-blocking socket connection
            // with the current client
            Err(_) => {
                println!("Connection to our server has been severed");
                break;
            }
        }

        // check if the server has received our client's message
        match receiver.try_recv() {
            Ok(msg) => {
                // store the current message into the buffer as a byte array
                let mut buffer = msg.clone().into_bytes();
                // complete those remained characters up to MSG_DIM with zeros
                buffer.resize(MSG_DIM, 0);

                // write message to socket
                client.write_all(&buffer)
                    .expect("Failed to write to socket");
                println!("The sent message is {:?}", msg);
            },
            // if the current channel is empty just continue
            Err(TryRecvError::Empty) => (),
            // elif the current channel has been disconnected just exit
            Err(TryRecvError::Disconnected) => break
        }

        // pause thread while being inactive
        sleep();
    });
}
