use std::{
    fs::File,
    io::{Read, Write},
};

use bincode::Options;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};

use crate::Function;

pub fn save_bytecode(function: &Function, filename: &str) -> std::io::Result<()> {
    let file = File::create(filename)?;

    let mut encoder = GzEncoder::new(file, Compression::best());
    let config = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .allow_trailing_bytes();

    let encoded = config
        .serialize(function)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    encoder.write_all(&encoded)?;
    encoder.finish()?;
    println!("Bytecode salvo");
    Ok(())
}

pub fn load_bytecode(filename: &str) -> std::io::Result<Function> {
    let file = File::open(filename)?;

    let mut decoder = GzDecoder::new(file);
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    let config = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .allow_trailing_bytes();

    let function: Function = config
        .deserialize(&buffer)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    Ok(function)
}
