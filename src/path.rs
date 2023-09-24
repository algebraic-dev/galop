use std::path::PathBuf;

use sha1::Sha1;
use sha1::Digest;

#[derive(Clone)]
pub struct Id(pub String);

/// Generates a new name for a repository
pub fn generate_destination(destination_path: PathBuf, name: Id) -> PathBuf {
    let mut path_buf = destination_path.clone();
    path_buf.push(name.0);

    path_buf
}

pub fn get_id(name: &str) -> Id {
    let mut hasher = Sha1::new();
    hasher.update(name);
    let result = hasher.finalize();
    let hex = format!("{:x}", result);
    Id(hex)
}