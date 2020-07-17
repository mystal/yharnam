use std::env;
use std::error::Error;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use prost::Message;
use serde::Deserialize;

use yarn_runner::yarn;

#[derive(Debug, Deserialize)]
pub struct Record {
    id: String,
    text: String,
    file: String,
    node: String,
    #[serde(rename="lineNumber")]
    line_number: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Read first argument as a path to a yarnc file.
    let proto_path = env::args().nth(1)
        .unwrap();
    let proto_path = PathBuf::from(proto_path);

    // Read the file's bytes and load a Program.
    let proto_data = fs::read(&proto_path)?;
    let program = yarn::Program::decode(&mut Cursor::new(&proto_data))?;
    println!("{:#?}", &program);

    // Load Records from a csv file.
    let mut csv_path = proto_path;
    csv_path.set_extension("csv");
    let mut csv_reader = csv::Reader::from_path(csv_path)?;
    let records: Vec<Record> = csv_reader.deserialize()
        .map(|result| result.unwrap())
        .collect();
    // dbg!(records);

    // TODO: Run the virtual machine!
    // TODO: Find the Start node.
    if let Some(node) = program.nodes.get("Start") {
    } else {
        eprintln!("Could not find Start node");
    }

    Ok(())
}
