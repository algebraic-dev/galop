// Executable that is useful to manipulate docker files and track some information about them. 
// It clones a single directory, creates a image with it, creates a container and then runs a single
// command to test then and extract metrics.

use std::{path::PathBuf, fs, time::Duration, io::Write};

use clap::Parser;
use galop::analyze::Return;
use tokio::time::timeout;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Args {
    Init,
    Run {
        file: String,

        #[arg(short, long, default_value_t = {"/media/ramdisk".to_string()})]
        git_installation: String,
    }
}

async fn run_participant(url: String, git_installation: String) {
    let id = galop::path::get_id(&url);

    let mut r = PathBuf::from("./submissions");
    r.push(format!("{}.json", id.0));

    if r.exists() {
        println!("[info] already {}", url); 
    } else {
        let analysis = galop::analyze(url.clone(), git_installation).await;

        let mut res = fs::File::create(r).unwrap();
    
        match analysis {
            Ok(result) => {
                let json = serde_json::to_string(&Return::Ok(result.maps)).unwrap();
                res.write_all(json.as_bytes()).unwrap();
            },
            Err(err) => {
                let json = serde_json::to_string(&Return::Err(err)).unwrap();
                res.write_all(json.as_bytes()).unwrap();
            },
        }
    }
}

#[tokio::main]
async fn main() {    
    let args = Args::parse();

    match args {
        Args::Init => {

        }
        Args::Run { git_installation, file } => {
            let url = "https://github.com/irbp/darinha.git".to_string();
            let part = run_participant(url, git_installation);

        }
    }

    
}
