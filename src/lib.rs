extern crate timer;
extern crate chrono;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration as StdDuration;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;

use timer::Timer;
use chrono::Duration;

pub mod constants;

pub struct BlockCache {
    block_map: HashMap<String,i32>
}

impl BlockCache {
    pub fn new(block_file_name: &str) -> Self {
        let block_map: HashMap<String, i32> = HashMap::new();
        todo!()
    }

    pub fn get_block(&self, ip: &str, user_agent: &str) -> u32 {
        todo!()
    }
}

pub fn foo() {
    let timer = Timer::new();

    // Schedule a repeating callback every 250 ms
    let guard = {
        timer.schedule_repeating(Duration::milliseconds(constants::BLOCK_FILE_TIMER_INTERVAL_MS), move || {
            match bar() {
                Ok(parts) => {

                    for s in &parts {
                        for k in s {
                            println!("  Subpart: {}", k);
                        }
                    }
                    println!("Bar output done");
                    std::io::stdout().flush().unwrap();
                }
                Err(e) => {
                    println!("Error from bar(): {}", e);
                    std::io::stdout().flush().unwrap();
                }
            }
        })
    };

    thread::sleep(StdDuration::from_secs(30));

    drop(guard);
    thread::sleep(StdDuration::from_millis(100));
}

pub fn bar() -> Result<Vec<Vec<String>>, std::io::Error> {
    let file = match File::open("sample_block_file.txt") {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening file: {}", err);
            return Err(err);
        }
    };
    
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let mut parts: Vec<Vec<String>> = Vec::new();
    
    loop {
        buffer.clear(); 
        match reader.read_line(&mut buffer) {
            Ok(0) => return Ok(parts),
            Ok(_) => {
                let part = buffer.trim().split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                parts.push(part);
            }
            Err(err) => {
                println!("Error reading line {}", err);
                return Err(err);
            }
        }
    }
}