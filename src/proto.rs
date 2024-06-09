use crate::{Error, Result};
use prost_reflect::{DescriptorPool, MessageDescriptor};

/// This function searches the provided file descriptors for a message descriptor that matches the
/// provided type URL.
///
/// # Errors
/// This function will return an error if the type URL is invalid or if no matching message
/// descriptor can be found.
pub fn search_file_descriptors(
    file_descriptor_pools: &[DescriptorPool],
    type_url: &str,
) -> Result<MessageDescriptor> {
    let message_name = message_name_from_type_url(type_url)?;

    file_descriptor_pools
        .iter()
        .find_map(|pool| pool.get_message_by_name(message_name))
        .ok_or_else(|| Error::InvalidTypeUrl(message_name.into()))
}

fn message_name_from_type_url(type_url: &str) -> Result<&str> {
    type_url
        .split('/')
        .nth(1)
        .ok_or_else(|| Error::InvalidTypeUrl(type_url.into()))
}
