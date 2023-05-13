use clap::Parser;
use serde_json::json;
use std::{collections::HashMap, fs::File, num::NonZeroUsize, path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]
pub struct Dictionary(pub HashMap<Vec<u8>, Vec<u8>>);

impl FromStr for Dictionary {
    type Err = String;

    fn from_str(dictionary: &str) -> Result<Self, Self::Err> {
        if dictionary == "internal" {
            let serde_json::Value::Object(map) = include!("conversions.rs") else {
                        panic!("Incorrect conversions file")
                    };
            let mut final_map = HashMap::with_capacity(map.len());
            for (k, v) in map {
                let k = k.into_bytes();
                let serde_json::Value::String(v) = v else {
                        panic!("Incorrect conversions file")
                };
                let v = v.into_bytes();
                final_map.insert(k, v);
            }
            return Ok(Dictionary(final_map));
        };

        let serde_json::Value::Object(map) =
            serde_json::from_reader(File::open(dictionary).map_err(|err| format!("{err}"))?)
                .map_err(|err| format!("{err}"))? else {
                    return Err(format!("{dictionary} does not appear to contain a JSON object"))
                };
        let mut final_map = HashMap::with_capacity(map.len());
        for (k, v) in map {
            let k = k.into_bytes();
            let serde_json::Value::String(v) = v else {
                panic!("Error in JSON dictionary: One of the values appears to not be a string")
            };
            let v = v.into_bytes();
            final_map.insert(k, v);
        }
        Ok(Dictionary(final_map))
    }
}

#[derive(Debug, Parser)]
pub struct Cli {
    /// Where to write the result to, defaults to stdout
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    /// A JSON dictionary of word -> word mappings
    #[arg(short, long, value_name = "JSON", default_value = "internal")]
    pub dictionary: Dictionary,
    /// The number of threads to use, defaults to the number of physical cores in the system
    #[arg(short, long, value_name = "THREADS", default_value_t = num_cpus::get_physical().try_into().unwrap_or(1.try_into().unwrap()))]
    pub threads: NonZeroUsize,
    /// Where to read the text from
    pub input: PathBuf,
}
