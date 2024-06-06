fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("capnp")
        .file("capnp/example.capnp")
        .run()
        .expect("failed to compile capnp files");
}
