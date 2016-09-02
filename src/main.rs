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
        read_stdin_forever(&transmitter);
    });

    // This function "fails" when it's done; w/e
    let _ = write_stdout_forever(&receiver);
}

fn read_stdin_forever(transmitter: &Sender<[u8; 512]>) {
    let mut bytes: [u8; 512] = [0; 512];

    loop {
        let read_bytes = io::stdin().read(&mut bytes);
        match read_bytes {
            Ok(count) => {
                // Read signals end-of-input-stream with Ok(0)
                if count == 0 {
                    return;
                }

                let transfer = bytes;
                bytes = [0; 512];

                transmitter.send(transfer).unwrap();
            },

            // Ignore errors; if there will never be more bytes, it will Ok(0)
            Err(_) => (),
        }
    }
}

fn write_stdout_forever(receiver: &Receiver<[u8; 512]>) -> Result<(), WriteError> {
    loop {
        // If this fails, stdin hung up; no more data to write
        let bytes = try!(receiver.recv().map_err(|_| WriteError::EOF));

        // If this fails, stdout hung up; nothing is listening anymore
        try!(io::stdout().write_all(&bytes).map_err(|_| WriteError::HungUp));
    }
}
