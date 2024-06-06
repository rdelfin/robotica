use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    prost_build::Config::new()
        .file_descriptor_set_path(
            PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"))
                .join("file_descriptor_set.bin"),
        )
        .enable_type_names()
        .type_name_domain(["."], "type.googleapis.com")
        .compile_protos(&["proto/types.proto"], &["proto/"])?;
    Ok(())
}
