use std::{collections::HashMap, time::Instant};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Analysis {
    pub data: String,
    pub duration: u128
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "tag", content = "data")]
pub enum Return {
    Ok(HashMap<String, Analysis>),
    Err(String)
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
    
    pub fn register(&mut self, log: String) {
        let time = self.last.elapsed();
        if let Some(place) = log.find("@!") {
            let str = log.split_at(place).1;
            let (key, val) = str.split_at(log.find("::").unwrap());
            self.maps.insert(key[2..].to_string(), Analysis {
                data: val[2..].trim().to_owned(),
                duration: time.as_millis()
            });
        }
        self.last = Instant::now();
    }
}


