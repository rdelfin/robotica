use chrono::{DateTime, Utc};
use prost_reflect::SerializeOptions;
use robotica::Node;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a new node with the name "simple_sub".
    let node = Node::new("simple_sub").await?;

    // Create an untyped subscriber. Notice how we're getting the file descriptors from the
    // [`robotica_types`] crate. You can also generate these yourself when creating custom protobuf
    // types.
    let mut subscriber = node.subscribe_untyped("test_topic".to_string()).await?;
    // Returned message contains a [`prost_reflect::DynamicMessage`] which can be introspected
    // into.
    while let Ok(msg) = subscriber.recv().await {
        let system_time: SystemTime = msg
            .header
            .message_timestamp
            .expect("header timestamp not set")
            .try_into()?;
        let date_time: DateTime<Utc> = system_time.into();

        // To serialize into JSON, we recommend following the recomendations from the
        // [`prost_reflect`] crate.
        let mut serializer = serde_json::Serializer::new(vec![]);
        msg.message.serialize_with_options(
            &mut serializer,
            &SerializeOptions::new().use_proto_field_name(true),
        )?;
        println!(
            "Received: {} (sent at {})",
            String::from_utf8_lossy(&serializer.into_inner()),
            date_time.to_rfc3339()
        );
    }
    Ok(())
}
