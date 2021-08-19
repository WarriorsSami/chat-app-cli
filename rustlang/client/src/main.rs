use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;
use std::env;


const LOCAL: &str = "127.0.0.1:6000";
const MSG_DIM: usize = 512;

// pause the current thread
fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn main() {
    // get user name from cli
    let args: Vec<String> = env::args().collect();
    let user_name = &args[1];
    println!("User name: {}", user_name);

    // Initialize socket stream and put it run in the background
    let mut client = TcpStream::connect(LOCAL)
        .expect("Stream failed to connect");
    client.set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    // set an async channel for our process
    let (sender, receiver) = mpsc::channel::<String>();

    // initialize a channel to send our client messages
    thread::spawn(move || loop {
        let mut buffer = vec![0; MSG_DIM];

        // read the client message inside our buffer
        match client.read_exact(&mut buffer) {
            Ok(_) => {
                let msg = buffer.into_iter().take_while(|&x| x != 0)
                    .collect::<Vec<_>>();

                let msg_str = String::from_utf8(msg.clone());

                println!("The received message is {:?}\n=>{:?}", msg, msg_str);
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

    // Add interactivity support for users
    println!("Write a message:");
    loop {
        // read messages from stdin to buffer
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)
            .expect("Failed to read from stdin");

        // convert our buffer into a message
        let msg_content = buffer.trim().to_string();
        let mut msg = user_name.to_string();
        msg.push_str(": ");
        msg.push_str(&msg_content);

        // and try to send it to our sever (or exit at user's demand)
        if msg.contains(":quit") || msg.contains(":q") || sender.send(msg).is_err() {break}
    }
    println!("Bye bye!");
}
