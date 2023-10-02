use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use prost::Message;

use yharnam::*;

const DEFAULT_START_NODE_NAME: &str = "Start";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();

    // Read first argument as a path to a yarnc file.
    args.next();
    let proto_path = args.next().unwrap();
    let proto_path = PathBuf::from(proto_path);
    println!("Opening file at {:?}", proto_path);

    let start_node = args.next().unwrap_or(DEFAULT_START_NODE_NAME.to_string());

    // Read the file's bytes and load a Program.
    let proto_data = fs::read(&proto_path)?;
    let program = Program::decode(&*proto_data)?;
    // println!("{:#?}", &program);

    // Load LineInfos from a csv file.
    let mut lines_csv_path = proto_path.clone();
    lines_csv_path.set_file_name(format!(
        "{}-Lines.csv",
        lines_csv_path.file_stem().unwrap().to_str().unwrap()
    ));

    let string_table: Vec<LineInfo> = csv::Reader::from_path(lines_csv_path)?
        .deserialize()
        .map(|result| result.unwrap())
        .collect();

    // Load tags from a csv file.
    let mut tags_csv_path = proto_path;
    tags_csv_path.set_file_name(format!(
        "{}-Metadata.csv",
        tags_csv_path.file_stem().unwrap().to_str().unwrap()
    ));

    let tags_table: Vec<MetadataInfo> = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(tags_csv_path)?
        .deserialize()
        .map(|result| result.unwrap())
        .collect();

    // Run the virtual machine!
    let mut vm = VirtualMachine::new(program);
    if vm.program.nodes.contains_key(&start_node) {
        // Set the start node.
        vm.set_node(&start_node)?;

        // Start executing.
        loop {
            match vm.continue_dialogue()? {
                SuspendReason::Nop => {}
                SuspendReason::Line(line) => {
                    let text = string_table
                        .iter()
                        .find(|line_info| line_info.id == line.id)
                        .map(|line_info| &line_info.text);

                    let tags = tags_table
                        .iter()
                        .find(|metadata_info| metadata_info.id == line.id)
                        .map(|metadata_info| &metadata_info.tags)
                        .cloned()
                        .unwrap_or_else(|| Vec::new());

                    if let Some(text) = text {
                        println!("{text}, tagged {tags:?}");
                    } else {
                        // TODO: Could not find line, handle error.
                    }
                }
                SuspendReason::Options(options) => {
                    println!("== Choose option ==");
                    for (i, opt) in options.iter().enumerate() {
                        let text = string_table
                            .iter()
                            .find(|line_info| line_info.id == opt.line.id)
                            .map(|line_info| &line_info.text);
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
                    vm.set_selected_option(selection)?;
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
                SuspendReason::InvalidOption(option_name) => {
                    println!("INVALID OPTION: {option_name} is not an option. Please try again");
                    break;
                }
            }
        }
    } else {
        eprintln!("Could not find start node: {}", DEFAULT_START_NODE_NAME);
    }

    Ok(())
}
