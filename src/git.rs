use std::path::PathBuf;

use git2::{Repository, Error};

/// Clones the repository and gives it a name
pub fn clone_directory(git_url: &str, destination_path: PathBuf) -> Result<(), Error> {
    match Repository::clone_recurse(git_url, destination_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}
