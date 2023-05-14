#![recursion_limit = "180"]
mod cli;

use std::{
    fs::File,
    io::{self, stdin, stdout, BufWriter, Read, Write},
};

use clap::Parser;
use cli::{Cli, Dictionary};

fn main() -> io::Result<()> {
    let Cli {
        input,
        output,
        dictionary: Dictionary(dictionary),
        chunk,
    } = Cli::parse();
    let chunk = chunk.get();
    let mut input_file: Box<dyn Read> = if let Some(path) = input {
        Box::new(File::open(path)?)
    } else {
        Box::new(stdin().lock())
    };
    let output: Box<dyn Write> = if let Some(path) = output {
        Box::new(File::create(path)?)
    } else {
        Box::new(BufWriter::with_capacity(16 * 1024, stdout().lock()))
    };
    let mut output_writer = BufWriter::with_capacity(chunk * 2, output);
    let mut buf: Vec<u8> = Vec::with_capacity(chunk);
    loop {
        match input_file.as_mut().take(chunk as u64).read_to_end(&mut buf) {
            Ok(0) => {
                // EOF case
                // Handle possible word overflow
                buf.make_ascii_lowercase();
                let word = dictionary.get(&buf).unwrap_or(&buf);
                output_writer.write_all(word)?;
                break;
            }
            Ok(_) => {
                let mut start = 0;
                let mut idx = 0;
                while idx < buf.len() {
                    let current = buf[idx];
                    if !current.is_ascii_alphabetic() {
                        // Terminate word
                        if start < idx {
                            let word = &mut buf[start..idx];
                            word.make_ascii_lowercase();
                            // Get word from dictionary
                            let word = dictionary.get(word).map(|w| w.as_slice()).unwrap_or(word);
                            output_writer.write_all(word)?;
                        };
                        // Write terminator
                        output_writer.write_all(&[current])?;
                        start = idx + 1;
                    }
                    idx += 1;
                }
                buf.drain(..start);
            }
            Err(err) if matches!(err.kind(), std::io::ErrorKind::Interrupted) => continue,
            Err(err) => {
                panic!("{err}")
            }
        }
    }
    Ok(())
}
