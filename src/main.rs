use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{self, Read, Write};

const CHUNK_SIZE: usize = 512;

enum PipeError {
    HungUp,
}

fn main() {
    let (transmitter, receiver) = channel::<[u8; CHUNK_SIZE]>();

    // Read stdin forever on a separate thread as fast as possible
    thread::spawn(move|| {
        // Ignore result and exit thread if we're done reading
        let _ = read_stdin_forever(&transmitter);
    });

    // If we ever stop writing, we're done; ignore and exit
    let _ = write_stdout_forever(&receiver);
}

fn read_stdin_forever(transmitter: &Sender<[u8; CHUNK_SIZE]>) -> Result<(), PipeError> {
    let mut bytes: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

    loop {
        match io::stdin().read(&mut bytes) {
            // read() signals end-of-input-stream with Ok(0)
            Ok(0) => return Ok(()),

            // Any other number of bytes? send em along
            Ok(_) => {
                // If we have bytes to send, send them and clear the buffer
                // If sending fails, the receiver hung up; exit
                try!(transmitter.send(bytes).map_err(|_| PipeError::HungUp));
                bytes = [0; CHUNK_SIZE];
            },

            // Ignore errors; if there will never be more bytes, it will Ok(0)
            Err(_) => (),
        };
    }
}

fn write_stdout_forever(receiver: &Receiver<[u8; CHUNK_SIZE]>) -> Result<(), PipeError> {
    loop {
        // If this fails, stdin hung up; no more data to write
        let bytes = match receiver.recv() {
            Ok(data) => data,
            Err(_) => return Ok(()),
        };

        // If this fails, stdout hung up; nothing is listening anymore
        try!(io::stdout().write_all(&bytes).map_err(|_| PipeError::HungUp));
    }
}
