use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{self, Read, Write};

enum WriteError {
    EOF,
    HungUp,
}

fn main() {
    let (transmitter, receiver) = channel::<[u8; 512]>();

    thread::spawn(move|| {
        read_stdin(&transmitter);
    });

    // This function "fails" when it's done; w/e
    let _ = write_stdout(&receiver);
}

fn read_stdin(transmitter: &Sender<[u8; 512]>) {
    let mut bytes: [u8; 512] = [0; 512];
    let mut index = 0;

    for byte in io::stdin().bytes() {
        match byte {
            Ok(data) => {
                if index == 512 {
                    let transfer = bytes;
                    bytes = [0; 512];

                    transmitter.send(transfer).unwrap();
                    index = 0;
                }

                bytes[index] = data;
                index = index + 1;
            },

            // Ignore impossible bytes
            Err(_) => (),
        }
    }

    if index > 0 {
        transmitter.send(bytes).unwrap();
    }
}

fn write_stdout(receiver: &Receiver<[u8; 512]>) -> Result<(), WriteError> {
    loop {
        // If this fails, stdin hung up; no more data to write
        let bytes = try!(receiver.recv().map_err(|_| WriteError::EOF));

        // If this fails, stdout hung up; nothing is listening anymore
        try!(io::stdout().write_all(&bytes).map_err(|_| WriteError::HungUp));
    }
}
