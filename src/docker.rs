use std::{path::PathBuf, sync::Arc};

use docker_api::{opts::{ImageBuildOpts, ContainerCreateOpts, LogsOpts, ContainerPruneOpts, ContainerStopOpts}, Docker, models::ImageBuildChunk, conn::TtyChunk};

use crate::{path::Id, analyze::Report};
use futures_util::{StreamExt, lock::Mutex};

pub fn start(url: String) -> Docker {
    let docker = docker_api::Docker::new(url).unwrap();
    docker
}

pub async fn build_image(docker: &Docker, id: Id, path: PathBuf) -> Result<(), String> {
    let binding = docker.images();

    let params = &ImageBuildOpts::builder::<PathBuf>(path).tag(id.0).build();

    let mut stream = binding.build(params);

    let mut err = Vec::new();

    let mut already = false;

    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(output) => match output {
                ImageBuildChunk::Error {
                    error,
                    ..
                } => {
                    err.push(error.to_string())
                },
                ImageBuildChunk::Update { stream } => {
                    if already {
                        print!("\x1b[1A\x1b[K")
                    } else {    
                        already = true
                    }
                    println!("[info] docker: {}", stream.to_string().lines().next().unwrap_or_default());
                }
                _ => ()
            },
            Err(e) => return Err(e.to_string()),
        }
    }

    if err.len() > 0 {
        Err(err.join("\n"))
    } else {
        Ok(())
    }
}

#[derive(Default)]
pub struct Data {
    pub docker_id: Mutex<Option<docker_api::Id>>
}

pub async fn run_image(docker: &Docker, dir: String, id: Id, analysis: &mut Report, data: Arc<Data>) -> Result<Vec<String>, String> {
    let params = ContainerCreateOpts::builder()
        .image(id.0)
        .volumes([format!("{dir}/tests/source.rinha:/var/rinha/source.rinha"), 
                  format!("{dir}/tests/source.rinha.json:/var/rinha/source.rinha.json")].into_iter())
        .cpus(2.0)
        .memory(2147483648)
        .build();

    let containers = &docker.containers();
    let container = containers.create(&params).await.map_err(|x| format!("cannot create container '{}'", x.to_string()))?;

    let mut id = data.docker_id.lock().await;
    *id = Some(container.id().clone());

    let params = LogsOpts::builder().stdout(true).stderr(true).all().follow(true).build();

    let mut stream = container.logs(&params);
    
    println!("[info] starting the container");
    
    container.start().await.map_err(|_| "cannot start container".to_string())?;


    println!("[info] running the container");

    analysis.reset();

    let mut log_info = Vec::new();
    let mut err_info = Vec::new();

    while let Some(res) = stream.next().await {
        match res {
            Ok(res) => {
                match res {
                    TtyChunk::StdIn(_) => todo!(),
                    TtyChunk::StdOut(o) => {
                        let log = std::str::from_utf8(&o).unwrap().to_owned();
                        if !analysis.register(log.clone()) {
                            if !log.contains("@@!") {
                                log_info.push(log)
                            }
                        };
                    },
                    TtyChunk::StdErr(e) => {
                        let value = std::str::from_utf8(&e).unwrap().to_owned();
                        print!("[error] err {}", value.clone());
                        err_info.push(value);
                    },
                }
            },
            Err(e) => return Err(format!("cannot read logs '{}'", e.to_string())),
        }
    }

    println!("[logs] end container");

    let _ = container.stop(&ContainerStopOpts::builder().build()).await;

    docker_prune(docker).await;

    if err_info.len() > 0 {    
        Ok(err_info)
    } else {
        Ok(log_info)
    }
}

pub async fn docker_prune(docker: &Docker) {
    let _ = docker.containers().prune(&ContainerPruneOpts::builder().build()).await;
}