// Executable that is useful to manipulate docker files and track some information about them. 
// It clones a single directory, creates a image with it, creates a container and then runs a single
// command to test then and extract metrics.

use std::{path::PathBuf, fs, io::Write};

use clap::Parser;
use galop::analyze::{Return, Participant};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Args {
    Run {
        file: String,

        #[arg(short, long, default_value_t = {"/media/ramdisk".to_string()})]
        git_installation: String,
    }
}


async fn run_participant(url: String, git_installation: String, participant: Participant) -> Result<(), String> {
    let id = galop::path::get_id(&url);

    let mut r = PathBuf::from("./submissions");
    r.push(format!("{}.json", id.0));

    if r.exists() {
        println!("[info] already {}", url); 
    } else {
        let analysis = galop::analyze(url.clone(), git_installation).await;

        let mut res = fs::File::create(r).unwrap();
    
        match analysis {
            Ok((result, log)) => {
                let json = serde_json::to_string(&Return::Ok {
                    data: result.maps,
                    participant,
                    log,
                }).unwrap();
                res.write_all(json.as_bytes()).unwrap();
            },
            Err(err) => {
                println!("[error] {}", err); 
                let json = serde_json::to_string(&Return::Err(err, participant)).unwrap();
                res.write_all(json.as_bytes()).unwrap();
            },
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {    
    let args = Args::parse();

    match args {
        Args::Run { git_installation, file } => {
            let contents = fs::read_to_string(file);
            let participants: Vec<Participant> = serde_json::from_str(&contents.unwrap()).unwrap();
            
            let mut count = 0;

            for participant in participants {
                let res = run_participant(participant.repository.clone(), git_installation.clone(), participant).await;
                
                if res.is_ok() {
                    count += 1;
                }
            }

            println!("[info] ok {count}")

        }
    }
}
