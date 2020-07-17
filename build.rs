fn main() {
    prost_build::compile_protos(
        &["src/yarn_spinner.proto"],
        &["src/"],
    ).unwrap();
}
