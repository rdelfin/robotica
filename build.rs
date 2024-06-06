use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    prost_build::Config::new()
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"))
                .join("file_descriptor_set.bin"),
        )
        .compile_protos(&["proto/example.proto"], &["proto/"])?;
    Ok(())
}
