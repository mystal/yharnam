use std::env;
use std::error::Error;
use std::fs;
use std::io::Cursor;

use prost::Message;

use yarn_runner::yarn;

fn main() -> Result<(), Box<dyn Error>> {
    // Read first argument, try to open the file there. Try to load it as a yarnc protobuf.
    let f = env::args().nth(1)
        .unwrap();
    dbg!(&f);

    let data = fs::read(&f)?;
    let program = yarn::Program::decode(&mut Cursor::new(&data))?;
    dbg!(&program);

    // TODO: Run the virtual machine!

    Ok(())
}
