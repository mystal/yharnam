use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Cursor};
use std::path::PathBuf;

use prost::Message;

use yharnam::*;

const DEFAULT_START_NODE_NAME: &str = "Start";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();

    // Read first argument as a path to a yarnc file.
    args.next();
    let proto_path = args.next()
        .unwrap();
    let proto_path = PathBuf::from(proto_path);

    let start_node = args.next()
        .unwrap_or(DEFAULT_START_NODE_NAME.to_string());

    // Read the file's bytes and load a Program.
    let proto_data = fs::read(&proto_path)?;
    let program = Program::decode(&mut Cursor::new(&proto_data))?;
    // println!("{:#?}", &program);

    // Load Records from a csv file.
    let mut csv_path = proto_path;
    csv_path.set_extension("csv");
    let mut csv_reader = csv::Reader::from_path(csv_path)?;
    let string_table: Vec<Record> = csv_reader.deserialize()
        .map(|result| result.unwrap())
        .collect();

    // Run the virtual machine!
    let mut vm = VirtualMachine::new(program);
    if vm.program.nodes.contains_key(&start_node) {
        // Set the start node.
        vm.set_node(&start_node);

        // Start executing.
        loop {
            match vm.continue_dialogue() {
                SuspendReason::Line(line) => {
                    let text = string_table.iter()
                        .find(|record| record.id == line.id)
                        .map(|record| &record.text);
                    if let Some(text) = text {
                        println!("{}", text);
                    } else {
                        // TODO: Could not find line, handle error.
                    }
                }
                SuspendReason::Options(options) => {
                    println!("== Choose option ==");
                    for (i, opt) in options.iter().enumerate() {
                        let text = string_table.iter()
                            .find(|record| record.id == opt.line.id)
                            .map(|record| &record.text);
                        if let Some(text) = text {
                            println!("{}: {}", i, text);
                        } else {
                            // TODO: Could not find line, handle error.
                        }
                    }

                    // Block to accept input from player.
                    let mut selection = String::new();
                    io::stdin().read_line(&mut selection)?;
                    let selection: u32 = selection.trim().parse()?;
                    vm.set_selected_option(selection);
                }
                SuspendReason::Command(command_text) => {
                    println!("== Command: {} ==", command_text);
                }
                SuspendReason::NodeChange { start, end } => {
                    println!("== Node end: {} ==", end);
                    println!("== Node start: {} ==", start);
                }
                SuspendReason::DialogueComplete(last_node) => {
                    println!("== Node end: {} ==", last_node);
                    println!("== Dialogue complete ==");
                    break;
                }
            }
        }
    } else {
        eprintln!("Could not find start node: {}", DEFAULT_START_NODE_NAME);
    }

    Ok(())
}
