use std::thread;
use std::sync::mpsc::channel;
use std::io::{self, Read, Write};

fn main() {
    let (transmitter, receiver) = channel::<u8>();

    thread::spawn(move|| {
        for byte in io::stdin().bytes() {
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
