use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Cursor};
use std::path::PathBuf;

use prost::Message;

use yarn_runner::*;

const DEFAULT_START_NODE_NAME: &str = "shop";
// const DEFAULT_START_NODE_NAME: &str = "Start";

fn main() -> Result<(), Box<dyn Error>> {
    // Read first argument as a path to a yarnc file.
    let proto_path = env::args().nth(1)
        .unwrap();
    let proto_path = PathBuf::from(proto_path);

    // Read the file's bytes and load a Program.
    let proto_data = fs::read(&proto_path)?;
    let program = yarn::Program::decode(&mut Cursor::new(&proto_data))?;
    // println!("{:#?}", &program);

    // Load Records from a csv file.
    let mut csv_path = proto_path;
    csv_path.set_extension("csv");
    let mut csv_reader = csv::Reader::from_path(csv_path)?;
    let string_table: Vec<Record> = csv_reader.deserialize()
        .map(|result| result.unwrap())
        .collect();

    // Run the virtual machine!
    let mut vm = VirtualMachine::new(program, string_table);
    // TODO: Allow altering the start node name.
    if vm.program.nodes.contains_key(DEFAULT_START_NODE_NAME) {
        // Set the start node.
        vm.set_node(DEFAULT_START_NODE_NAME);

        // Start executing.
        vm.continue_dialogue();
        loop {
            match vm.execution_state {
                ExecutionState::Stopped => break,
                ExecutionState::Running => panic!(),
                ExecutionState::Suspended => vm.continue_dialogue(),
                ExecutionState::WaitingOnOptionSelection => {
                    // Block to accept input from player.
                    let mut selection = String::new();
                    io::stdin().read_line(&mut selection)?;
                    let selection: u32 = selection.trim().parse()?;
                    vm.set_selected_option(selection);
                }
            }
        }
    } else {
        eprintln!("Could not find start node: {}", DEFAULT_START_NODE_NAME);
    }

    Ok(())
}
