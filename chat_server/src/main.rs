use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let socket_address = "localhost:3000";
    let server = TcpListener::bind(socket_address)?;
    server.set_nonblocking(true)?;

    let mut clients = Vec::new();
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, address)) = server.accept() {
            println!("A client has connected from remote address: {:?}", address);
            clients.push(socket.try_clone()?);
            let tx = tx.clone();

            thread::spawn(move || loop {
                let mut buf = [0; 32];
                match socket.read_exact(&mut buf) {
                    Ok(_) => {
                        let msg = buf
                            .iter()
                            .take_while(|&x| *x != 0)
                            .map(|&x| x)
                            .collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Failed to parse to string");

                        println!("[{}]> {:?}", address, msg);
                        tx.send(msg).expect("Failed to send message over channel");
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with {:?}", address);
                        break;
                    }
                }
                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buf = msg.clone().into_bytes();
                    buf.resize(32, 0);
                    client.write_all(&buf).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(250));
}
