use std::path::PathBuf;

use docker_api::{opts::{ImageBuildOpts, ContainerCreateOpts, LogsOpts, ImagePruneOpts, ContainerPruneOpts, ContainerStopOpts}, Docker, models::ImageBuildChunk, conn::TtyChunk};


use crate::{path::Id, analyze::Report};
use futures_util::StreamExt;

pub fn start(url: String) -> Docker {
    docker_api::Docker::new(url).unwrap()
}

pub async fn build_image(docker: &Docker, id: Id, path: PathBuf) -> Result<(), String> {
    let binding = docker.images();

    let params = &ImageBuildOpts::builder::<PathBuf>(path).tag(id.0).build();

    let mut stream = binding.build(params);

    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(output) => match output {
                ImageBuildChunk::Error {
                    error,
                    ..
                } => return Err(error),
                ImageBuildChunk::Update { stream } => {
                    println!("[info] docker: {}", stream.to_string().trim_end());
                }
                _ => ()
            },
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(())
}

pub async fn run_image(docker: &Docker, id: Id, analysis: &mut Report) -> Result<(), String> {
    let params = ContainerCreateOpts::builder()
        .image(id.0)
        .volumes(["/home/sofia/Projects/rinha-tools/galop/tests/source.rinha:/var/rinha/source.rinha", 
                  "/home/sofia/Projects/rinha-tools/galop/tests/source.rinha.json:/var/rinha/source.rinha.json"].into_iter())
        .build();

    let containers = &docker.containers();
    let container = containers.create(&params).await.map_err(|x| format!("cannot create container '{}'", x.to_string()))?;

    let params = LogsOpts::builder().stdout(true).stderr(true).all().follow(true).build();

    let mut stream = container.logs(&params);
    
    container.start().await.map_err(|_| "cannot start container".to_string())?;
    analysis.reset();

    while let Some(res) = stream.next().await {
        match res {
            Ok(res) => {
                match res {
                    TtyChunk::StdIn(_) => todo!(),
                    TtyChunk::StdOut(o) => {
                        analysis.register(std::str::from_utf8(&o).unwrap().to_owned());
                    },
                    TtyChunk::StdErr(e) => print!("[error] err {}", std::str::from_utf8(&e).unwrap()),
                }
            },
            Err(e) => return Err(format!("cannot read logs '{}'", e.to_string())),
        }
    }

    println!("[logs] end container");

    let _ = container.stop(&ContainerStopOpts::builder().build()).await;

    docker_prune(docker).await;

    Ok(())
}

pub async fn docker_prune(docker: &Docker) {
    let _ = docker.containers().prune(&ContainerPruneOpts::builder().build()).await;
    let _ = docker.images().prune(&ImagePruneOpts::builder().build()).await;
}