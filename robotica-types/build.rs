fn main() -> anyhow::Result<()> {
    let mut config = prost_build::Config::new();
    config
        .enable_type_names()
        .type_name_domain(["."], "type.googleapis.com");

    prost_reflect_build::Builder::new()
        .file_descriptor_set_bytes("DESCRIPTOR_SET_BYTES")
        .configure(&mut config, &["proto/types.proto"], &["proto/"])?;

    config.compile_protos(&["proto/types.proto"], &["proto/"])?;
    Ok(())
}
