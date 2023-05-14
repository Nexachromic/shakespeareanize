#![recursion_limit = "180"]
mod cli;

use std::{
    fs::{File, OpenOptions},
    io::{self, stdout, Write},
    thread,
};

use clap::Parser;
use cli::{Cli, Dictionary};
use memmap2::MmapOptions;

fn main() -> io::Result<()> {
    let Cli {
        input,
        output,
        dictionary: Dictionary(dictionary),
        threads,
    } = Cli::parse();
    let mut threads = threads.get();
    let input = input.canonicalize()?;
    if output.as_ref() == Some(&input) {
        panic!("Input and output files cannot be identical!");
    }

    let mut output: Box<dyn Write> = if let Some(path) = output {
        Box::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(path)
                .expect("Failed to create output file"),
        )
    } else {
        Box::new(stdout())
    };
    let input_file = File::open(input).expect("Failed to open input file");
    let input_map = unsafe {
        MmapOptions::new()
            .populate()
            .map(&input_file)
            .map_err(|err| {
                eprintln!("Failed to map input file!");
                err
            })
            .unwrap()
    };
    const MIN_WORK: usize = 64 * 1024; // Minimum amount of work per-thread
    let max_threads = (input_map.len() / MIN_WORK).max(1);
    threads = threads.min(max_threads);
    let per_thread = input_map.len() / threads;

    // Getting min and max len of keys
    #[cfg(feature = "bounds-opt")]
    let mut min_len = usize::MAX;
    #[cfg(feature = "bounds-opt")]
    let mut max_len = 0;
    #[cfg(feature = "bounds-opt")]
    for key in dictionary.keys() {
        let len = key.len();
        min_len = min_len.min(len);
        max_len = max_len.max(len);
    }
    thread::scope(|s| {
        let dictionary = &dictionary;
        let mut thread_list = Vec::with_capacity(threads);
        let lastthread = threads - 1;
        for thread in 1..threads {
            let start = per_thread * thread;
            let end = if thread != lastthread {
                start + per_thread
            } else {
                input_map.len()
            };
            let thread_partition_upper = unsafe { input_map.get_unchecked(..end) };
            let thread = s.spawn(move || {
                // backtrack to byte after the first terminator before partition
                let backtrack = thread_partition_upper[..start]
                    .into_iter()
                    .rev()
                    .position(|b| !b.is_ascii_alphabetic())
                    .unwrap_or_else(|| {
                        if thread != lastthread
                            // If our first char is a terminator, and no thread before us got a
                            // terminator, we need to capture it
                            && thread_partition_upper[start].is_ascii_alphabetic()
                        {
                            0
                        } else {
                            // If there are no word separators before our thread's partition, and we're the last thread, input is a single word and we need to process all of it
                            start
                        }
                    });
                let thread_partition = &thread_partition_upper[start - backtrack..];

                let mut out = Vec::with_capacity(thread_partition.len());
                let mut start = 0;
                for (idx, &byte) in thread_partition.into_iter().enumerate() {
                    if !byte.is_ascii_alphabetic() {
                        // Terminate word
                        if start < idx {
                            let original_word = &thread_partition[start..idx];
                            let len = out.len();
                            // Add the word to the buffer, so we can convert it to lowercase
                            out.extend_from_slice(original_word);
                            #[cfg(feature = "bounds-opt")]
                            if !(original_word.len() <= max_len && out.len() >= min_len) {
                                // If the word does not fit in the length bounds of the words in
                                // the dictionary
                                start = idx + 1;
                                out.push(byte);
                                continue;
                            }
                            let word = &mut out[len..];
                            word.make_ascii_lowercase();
                            // Get the new word
                            let new_word = dictionary
                                .get(word)
                                .map(|w| w.as_slice())
                                .unwrap_or(original_word);
                            // Remove the word we converted to lowercase from the output buffer
                            out.truncate(len);
                            // Add the converted/original word into the output buffer
                            out.extend_from_slice(new_word);
                        }
                        // Write terminating byte
                        start = idx + 1;
                        out.push(byte)
                    }
                }
                // Ignore word overflow unless we're the very last thread as otherwise, the
                // overflow will be picked up by the following thread
                if thread == lastthread {
                    let original_word = &thread_partition[start..];
                    let len = out.len();
                    // Add the word to the buffer, so we can convert it to lowercase
                    out.extend_from_slice(original_word);
                    let word = &mut out[len..];
                    word.make_ascii_lowercase();
                    // Get the new word
                    let new_word = dictionary
                        .get(word)
                        .map(|w| w.as_slice())
                        .unwrap_or(original_word);
                    // Remove the word we converted to lowercase from the output buffer
                    out.truncate(len);
                    // Add the converted/original word into the output buffer
                    out.extend_from_slice(new_word);
                }
                out
            });
            thread_list.push(thread)
        }
        let first_out = {
            let thread_partition = unsafe { input_map.get_unchecked(0..per_thread) };
            // Use main as first "thread"
            let mut out = Vec::with_capacity(thread_partition.len());
            let mut start = 0;
            for (idx, &byte) in thread_partition.into_iter().enumerate() {
                if !byte.is_ascii_alphabetic() {
                    // Terminate word
                    if start < idx {
                        let original_word = &thread_partition[start..idx];
                        let len = out.len();
                        // Add the word to the buffer, so we can convert it to lowercase
                        out.extend_from_slice(original_word);
                        let word = &mut out[len..];
                        word.make_ascii_lowercase();
                        // Get the new word
                        let new_word = dictionary
                            .get(word)
                            .map(|w| w.as_slice())
                            .unwrap_or(original_word);
                        // Remove the word we converted to lowercase from the output buffer
                        out.truncate(len);
                        // Add the converted/original word into the output buffer
                        out.extend_from_slice(new_word);
                    }
                    // Write terminating byte
                    start = idx + 1;
                    out.push(byte)
                }
            }
            // Ignore word overflow if there's multiple threads as it'll be handled by the following thread
            if threads == 1 {
                let original_word = &thread_partition[start..];
                let len = out.len();
                // Add the word to the buffer, so we can convert it to lowercase
                out.extend_from_slice(original_word);
                let word = &mut out[len..];
                word.make_ascii_lowercase();
                // Get the new word
                let new_word = dictionary
                    .get(word)
                    .map(|w| w.as_slice())
                    .unwrap_or(original_word);
                // Remove the word we converted to lowercase from the output buffer
                out.truncate(len);
                // Add the converted/original word into the output buffer
                out.extend_from_slice(new_word);
            }
            out
        };
        output
            .write_all(&first_out)
            .map_err(|err| {
                eprintln!("Failed to write to output file");
                err
            })
            .unwrap();
        for thread in thread_list {
            let finished = thread
                .join()
                .expect("Failed to join thread, something really went wrong!");
            output
                .write_all(&finished)
                .map_err(|err| {
                    eprintln!("Failed to write to output file");
                    err
                })
                .unwrap();
        }
    });
    Ok(())
}
