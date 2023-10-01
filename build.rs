use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/yarn_spinner.proto"], &["src/"])?;

    Ok(())
}
