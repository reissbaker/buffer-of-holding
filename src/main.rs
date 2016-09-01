use std::thread;
use std::sync::mpsc::channel;
use std::io::{self, Read, Write};

fn main() {
    let (transmitter, receiver) = channel::<u8>();

    thread::spawn(move|| {
        for byte in io::stdin().bytes() {
            // TODO: r u freaking kidding me with these unwraps
            // don't send a byte if the byte read fails duh
            transmitter.send(byte.unwrap()).unwrap();
        }
    });

    let mut dead = false;
    while !dead {
        let byte = receiver.recv();
        match byte {
            Ok(data) => {
                // TODO: buffer data when write fails
                io::stdout().write(&[data]);
            },
            Err(_) => {
                dead = true;
            },
        }
    }
}
