//! Executable that is useful to manipulate docker files and track some information about them. 
//! It clones a single directory, creates a image with it, creates a container and then runs a single
//! command to test then and extract metrics.

pub mod analyze;
pub mod path;
pub mod git;
pub mod docker;

use std::{path::PathBuf, fs, time::Duration, sync::Arc};

use analyze::Report;
use docker::Data;
use docker_api::opts::ContainerStopOptsBuilder;
use path::Id;
use tokio::time::timeout;

use crate::docker::{build_image, run_image};

pub fn clone_repo(url: String, git_directory: PathBuf) -> Result<(Id, PathBuf), String> {

    // The destination path for git repositories. It's usually in memory
    let git_installation = PathBuf::from(git_directory);

    let id = crate::path::get_id(&url);
    let folder_destination = crate::path::generate_destination(git_installation, id.clone());
    
    crate::git::clone_directory(&url, folder_destination.clone()).map_err(|e| format!("cannot clone repository '{}'", e.message()))?;

    Ok((id, folder_destination))
}

pub fn cleanup_git(folder: PathBuf) -> Result<(), String> {
    fs::remove_dir_all(folder.clone()).map_err(|_| "cannot remove repository")?;
    Ok(())
}

pub async fn run_repository(url: String, dir: String, git_installation: String, analysis: &mut Report, data: Arc<Data>) -> Result<Vec<String>, String> {
    let docker = crate::docker::start("tcp://127.0.0.1:2375".to_string());

    let (id, folder) = clone_repo(url.to_string(), git_installation.into())?;
    println!("[info] cloned the directory to {:?}", folder.clone());

    build_image(&docker, id.clone(), folder.clone()).await?;
    println!("[info] build the image");

    let log = run_image(&docker, dir, id.clone(), analysis, data).await?;

    let _ = cleanup_git(folder.clone());
    println!("[info] deleted the directory to {:?}", folder);
    
    Ok(log)
}

pub async fn analyze(url: String, dir: String, git_installation: String) -> Result<(Report, Vec<String>), String> {
    let mut analysis = Report::start();

    let data = Arc::new(Data::default());
    
    match timeout(Duration::from_secs(60*10), run_repository(url, dir, git_installation, &mut analysis, data.clone())).await {
        Err(_) => {
            let docker = crate::docker::start("tcp://127.0.0.1:2375".to_string());
        
            let mutex_guard = data.docker_id.lock().await;
        
            if let Some(id) = mutex_guard.clone() {
                let _ = docker.containers().get(id).stop(&ContainerStopOptsBuilder::default().build()).await;
            }

            analysis.register("@!timeout::".to_owned());
            Ok((analysis, Default::default()))
        },
        Ok(ok) => ok.map(|x| (analysis, x))
    }
}