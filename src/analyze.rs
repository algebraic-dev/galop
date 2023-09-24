use std::{collections::HashMap, time::Instant};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Analysis {
    pub data: String,
    pub duration: u128
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Participant {
    pub name: String,
    pub repository: String,
    pub language: String,
    pub r#type: String,
    pub social: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "tag", content = "data")]
pub enum Return {
    Ok {
        data: HashMap<String, Analysis>,
        participant: Participant,
        log: Vec<String> 
    },
    Err(String, Participant)
}

#[derive(Debug)]
pub struct Report {
    pub maps: HashMap<String, Analysis>,
    pub last: Instant,
}

impl Report {
    pub fn start() -> Report {
        Report {
            maps: Default::default(),
            last: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.last = Instant::now();
    }
    
    pub fn register(&mut self, log: String) -> bool {
        let time = self.last.elapsed();
        if let Some(place) = log.find("@!") {
            let str = log.split_at(place).1;
            let (key, val) = str.split_at(log.find("::").unwrap());
            self.maps.insert(key[2..].to_string(), Analysis {
                data: val[2..].trim().to_owned(),
                duration: time.as_micros()
            });
            self.last = Instant::now();
            true
        } else {
            false
        }
       
    }
}


