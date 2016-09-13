use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::io::{self, Read, Write};

// The size in bytes that we read per read() call from stdin
const CHUNK_SIZE: usize = 512;

enum PipeError { HungUp, }
type PipeResult = Result<(), PipeError>;
type SizedBuffer = ([u8; CHUNK_SIZE], usize);

fn main() {
    let (transmitter, receiver) = channel::<SizedBuffer>();

    // Read stdin forever on a separate thread as fast as possible
    thread::spawn(move|| {
        // Ignore result and exit thread if we're done reading
        let _ = read_stdin_forever(&transmitter);
    });

    // If we ever stop writing, we're done; ignore and exit
    let _ = write_stdout_forever(&receiver);
}

fn read_stdin_forever(transmitter: &Sender<SizedBuffer>) -> PipeResult {
    let mut bytes: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

    loop {
        match io::stdin().read(&mut bytes) {
            // read() signals end-of-input-stream with Ok(0)
            Ok(0) => return Ok(()),

            // Any other number of bytes? send em along
            Ok(n) => {
                // If we have bytes to send, send them and clear the buffer
                // If sending fails, the receiver hung up; exit
                let sized_buf = (bytes, n);
                try!(transmitter.send(sized_buf).map_err(|_| PipeError::HungUp));
                bytes = [0; CHUNK_SIZE];
            },

            // Ignore errors; if there will never be more bytes, it will Ok(0)
            Err(_) => (),
        };
    }
}

fn write_stdout_forever(receiver: &Receiver<SizedBuffer>) -> PipeResult {
    loop {
        // If this fails, stdin hung up; no more data to write
        let (bytes, size) = match receiver.recv() {
            Ok(data) => data,
            Err(_) => return Ok(()),
        };

        // Grab only the bytes we actually read from the buffer.
        // We do this here rather than in the reader to keep *all* CPU activity
        // on the writer thread, freeing up the reader to read as quickly as
        // possible.
        let read_bytes: Vec<u8> = bytes.into_iter()
                              .take(size)
                              .map(|byte_addr| *byte_addr)
                              .collect();

        // If this fails, stdout hung up; nothing is listening anymore
        try!(io::stdout().write_all(&read_bytes).map_err(|_| PipeError::HungUp));
    }
}
