use std::{collections::{HashMap}, fs::File, io::{BufReader, Cursor}};

use crate::{Source, source::Buffered};

type SampleBuffer =  Buffered<crate::Decoder<BufReader<File>>>;

pub struct SampleBank {
    samples: HashMap<String, SampleBuffer>
}

impl SampleBank {
    pub fn new(&self) -> Self {
        SampleBank {
            samples: HashMap::new()
        }
    }

    pub fn read_file(&self, key: &str, path: &str){
        match File::open(path) {
            Ok(file) => {
                let buf=crate::Decoder::new(
                    BufReader::new(file)
                ).unwrap().buffered();
                self.samples.insert(key.to_string(), buf);
            },
            Err(_) => println!("file not found!"),
        }
    }
    // https://rustwasm.github.io/docs/wasm-bindgen/reference/types/number-slices.html
    // no^ we are decoding here, so makes sense to just give string
    pub fn read_bytes(&self, key: &str, data: &str){
        let buf=crate::Decoder::new(
            BufReader::new(Cursor::new(data))
        ).unwrap().buffered();
        self.samples.insert(key.to_string(), buf);
    }
}
