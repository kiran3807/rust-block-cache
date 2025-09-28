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
use indexmap::IndexSet;

pub mod constants;

pub struct BlockCache {
    block_map: Arc<Mutex<HashMap<u64, String>>>, // Key: combined numeric key -> Value: second field
    ip_set: Arc<Mutex<IndexSet<String>>>, // All unique IP addresses
    user_agent_set: Arc<Mutex<IndexSet<String>>>, // All unique User Agents
    _guard: Arc<Mutex<Option<timer::Guard>>> // Keep timer alive
}

impl BlockCache {
    pub fn new(block_file_name: &str) -> Self {
        let block_map = Arc::new(Mutex::new(HashMap::new()));
        let ip_set = Arc::new(Mutex::new(IndexSet::new()));
        let user_agent_set = Arc::new(Mutex::new(IndexSet::new()));
        let guard_holder = Arc::new(Mutex::new(None));
        
        // Clone references for the timer callback
        let block_map_clone = Arc::clone(&block_map);
        let ip_set_clone = Arc::clone(&ip_set);
        let user_agent_set_clone = Arc::clone(&user_agent_set);
        let file_name = block_file_name.to_string();
        
        let timer = Timer::new();
        let guard = timer.schedule_repeating(Duration::milliseconds(5000), move || {
            // Update HashMap and IndexSets every 5 seconds
            Self::update_collections(&block_map_clone, &ip_set_clone, &user_agent_set_clone, &file_name);
        });
        
        // Store the guard to keep timer alive
        *guard_holder.lock().unwrap() = Some(guard);
        
        // Initial load
        Self::update_collections(&block_map, &ip_set, &user_agent_set, block_file_name);
        
        BlockCache {
            block_map,
            ip_set,
            user_agent_set,
            _guard: guard_holder,
        }
    }
    
    fn update_collections(
        block_map: &Arc<Mutex<HashMap<u64, String>>>, 
        ip_set: &Arc<Mutex<IndexSet<String>>>,
        user_agent_set: &Arc<Mutex<IndexSet<String>>>,
        file_name: &str
    ) {
        let file = match File::open(file_name) {
            Ok(file) => file,
            Err(err) => {
                println!("Error opening file {}: {}", file_name, err);
                return;
            }
        };
        
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        let mut new_ip_set = IndexSet::new();
        let mut new_user_agent_set = IndexSet::new();
        let mut new_map = HashMap::new();
        
        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let parts: Vec<&str> = buffer.trim().split(',').collect();
                    
                    if parts.len() >= 2 {
                        let ip = parts[0].trim().to_string();
                        let second_field = parts[1].trim().to_string();
                        let user_agent = if parts.len() >= 3 { 
                            parts[2].trim().to_string() 
                        } else { 
                            String::new() 
                        };
                        
                        if !ip.is_empty() && !second_field.is_empty() {
                            // Add IP to IndexSet and get its index
                            let (ip_index, _) = new_ip_set.insert_full(ip);
                            
                            let key = if !user_agent.is_empty() {
                                // Add User Agent to IndexSet and get its index
                                let (ua_index, _) = new_user_agent_set.insert_full(user_agent);
                                // Combine IP index with UA index: (ip_index << 32) | ua_index
                                ((ip_index as u64) << 32) | (ua_index as u64)
                            } else {
                                // Use IP index directly
                                ip_index as u64
                            };
                            
                            // Insert into HashMap immediately
                            new_map.insert(key, second_field);
                        }
                    }
                }
                Err(err) => {
                    println!("Error reading line: {}", err);
                    break;
                }
            }
        }
        
        // Update all shared collections
        let mut map = block_map.lock().unwrap();
        let mut ips = ip_set.lock().unwrap();
        let mut user_agents = user_agent_set.lock().unwrap();
        
        *map = new_map;
        *ips = new_ip_set;
        *user_agents = new_user_agent_set;
        
        println!("Updated collections: {} IPs, {} User Agents, {} mappings", 
                 ips.len(), user_agents.len(), map.len());
    }

    pub fn get_block(&self, ip: &str, user_agent: &str) -> u32 {
        let map = self.block_map.lock().unwrap();
        let ips = self.ip_set.lock().unwrap();
        let user_agents = self.user_agent_set.lock().unwrap();
        
        // Find IP index in IndexSet
        let ip_index = match ips.get_index_of(ip) {
            Some(index) => index,
            None => return 0, // IP not found in IndexSet
        };
        
        // Try to find the key
        let key = if !user_agent.is_empty() {
            // Try to find user agent index
            if let Some(ua_index) = user_agents.get_index_of(user_agent) {
                ((ip_index as u64) << 32) | (ua_index as u64)
            } else {
                return 0; // User agent not found
            }
        } else {
            ip_index as u64
        };
        
        match map.get(&key) {
            Some(_) => 1, // Match found
            None => 0,    // Not found
        }
    }
    
    pub fn print_cache_contents(&self) {
        let map = self.block_map.lock().unwrap();
        let ips = self.ip_set.lock().unwrap();
        let user_agents = self.user_agent_set.lock().unwrap();
        
        println!("=== BlockCache Contents ===");
        
        // Print IP IndexSet separately
        println!("\n1. IP Address IndexSet ({} entries):", ips.len());
        println!("   Index | IP Address");
        println!("   ------|----------");
        if ips.is_empty() {
            println!("   (empty)");
        } else {
            for (index, ip) in ips.iter().enumerate() {
                println!("   {:5} | {}", index, ip);
            }
        }
        
        // Print User Agent IndexSet separately
        println!("\n2. User Agent IndexSet ({} entries):", user_agents.len());
        println!("   Index | User Agent");
        println!("   ------|----------");
        if user_agents.is_empty() {
            println!("   (empty)");
        } else {
            for (index, user_agent) in user_agents.iter().enumerate() {
                println!("   {:5} | {}", index, user_agent);
            }
        }
        
        // Print HashMap separately
        println!("\n3. HashMap - Index-based Key Mappings ({} entries):", map.len());
        println!("   Numeric Key | Decoded Key | Second Field");
        println!("   ------------|-------------|-------------");
        if map.is_empty() {
            println!("   (empty)");
        } else {
            for (key, second_field) in map.iter() {
                let ip_index = (key >> 32) as usize;
                let ua_index = (key & 0xFFFFFFFF) as usize;
                
                if ua_index == 0 && ip_index == (*key as usize) {
                    // IP index only key
                    let invalid_ip = "INVALID".to_string();
                    let ip_addr = ips.get_index(ip_index).unwrap_or(&invalid_ip);
                    println!("   {:11} | IP_idx: {} ({}) | {}", key, ip_index, ip_addr, second_field);
                } else {
                    // Combined key (IP index + UA index)
                    let invalid_ip = "INVALID".to_string();
                    let invalid_ua = "INVALID".to_string();
                    let ip_addr = ips.get_index(ip_index).unwrap_or(&invalid_ip);
                    let ua_str = user_agents.get_index(ua_index).unwrap_or(&invalid_ua);
                    println!("   {:11} | IP_idx: {} ({}), UA_idx: {} ({}) | {}", 
                             key, ip_index, ip_addr, ua_index, ua_str, second_field);
                }
            }
        }
        
        println!("\n==========================");
    }
}