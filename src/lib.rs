extern crate timer;
extern crate chrono;

use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::collections::HashMap;

use timer::Timer;
use chrono::Duration;
use indexmap::IndexSet;
use arc_swap::ArcSwap;

pub mod constants;

// Struct to hold all cache data together for atomic updates
#[derive(Clone)]
pub struct CacheData {
    pub block_map: HashMap<u64, String>, // Key: combined numeric key -> Value: second field
    pub ip_set: IndexSet<String>, // All unique IP addresses
    pub user_agent_set: IndexSet<String>, // All unique User Agents
}

pub struct BlockCache {
    cache_data: Arc<ArcSwap<CacheData>>, // All data wrapped in ArcSwap for lock-free reads
    _guard: Arc<Mutex<Option<timer::Guard>>> // Keep timer alive
}

impl BlockCache {
    pub fn new(block_file_name: &str) -> Self {
        // Create initial empty cache data
        let initial_data = CacheData {
            block_map: HashMap::new(),
            ip_set: IndexSet::new(),
            user_agent_set: IndexSet::new(),
        };
        
        let cache_data = Arc::new(ArcSwap::new(Arc::new(initial_data)));
        let guard_holder = Arc::new(Mutex::new(None));
        
        // Clone for timer callback
        let cache_data_clone = Arc::clone(&cache_data);
        let file_name = block_file_name.to_string();
        
        let timer = Timer::new();
        let guard = timer.schedule_repeating(Duration::milliseconds(5000), move || {
            // Update all collections every 5 seconds
            Self::update_collections(&cache_data_clone, &file_name);
        });
        
        // Store the guard to keep timer alive
        *guard_holder.lock().unwrap() = Some(guard);
        
        // Initial load
        Self::update_collections(&cache_data, block_file_name);
        
        BlockCache {
            cache_data,
            _guard: guard_holder,
        }
    }
    
    fn update_collections(cache_data: &Arc<ArcSwap<CacheData>>, file_name: &str) {
        println!("Updating collections from file: {}", file_name);
        let _ = io::stdout().flush();
        
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
        
        // Get sizes before moving into Arc
        let ip_count = new_ip_set.len();
        let user_agent_count = new_user_agent_set.len();
        let mapping_count = new_map.len();
        
        // Create new cache data instance
        let new_cache = Arc::new(CacheData {
            block_map: new_map,
            ip_set: new_ip_set,
            user_agent_set: new_user_agent_set,
        });
        
        // Atomically update with new data
        cache_data.store(new_cache);
        
        println!("Updated collections: {} IPs, {} User Agents, {} mappings", 
                 ip_count, user_agent_count, mapping_count);
        let _ = io::stdout().flush();
    }

    pub fn get_block(&self, ip: &str, user_agent: &str) -> Result<String, String> {
        // Load current cache data snapshot
        let cache = self.cache_data.load();
        
        // Step 1: Check if IP exists in the IP set
        let ip_index = match cache.ip_set.get_index_of(ip) {
            Some(index) => index,
            None => return Ok("0".to_string()), // IP not found, return 0 immediately
        };
        
        // Step 2: Check if there's a single IP index entry in the hashmap
        let ip_only_key = ip_index as u64;
        if let Some(value) = cache.block_map.get(&ip_only_key) {
            return Ok(value.clone()); // Found single IP entry, return its value
        }
        
        // Step 3: Check if user agent exists in the user agent set
        let ua_index = match cache.user_agent_set.get_index_of(user_agent) {
            Some(index) => index,
            None => return Ok("0".to_string()), // User agent not found, return 0 immediately
        };
        
        // Step 4: Create combined key and search for it
        let combined_key = ((ip_index as u64) << 32) | (ua_index as u64);
        match cache.block_map.get(&combined_key) {
            Some(value) => Ok(value.clone()), // Found combined entry, return its value
            None => Err(format!(
                "Anomalous state: IP '{}' (index {}) and User Agent '{}' (index {}) exist in sets but combined key {} not found in hashmap",
                ip, ip_index, user_agent, ua_index, combined_key
            )), // This should not happen - anomalous state
        }
    }
    
    pub fn print_cache_contents(&self) {
        // Load current cache data snapshot (lock-free read!)
        let cache = self.cache_data.load();
        
        println!("=== BlockCache Contents ===");
        
        // Print IP IndexSet separately
        println!("\n1. IP Address IndexSet ({} entries):", cache.ip_set.len());
        println!("   Index | IP Address");
        println!("   ------|----------");
        if cache.ip_set.is_empty() {
            println!("   (empty)");
        } else {
            for (index, ip) in cache.ip_set.iter().enumerate() {
                println!("   {:5} | {}", index, ip);
            }
        }
        
        // Print User Agent IndexSet separately
        println!("\n2. User Agent IndexSet ({} entries):", cache.user_agent_set.len());
        println!("   Index | User Agent");
        println!("   ------|----------");
        if cache.user_agent_set.is_empty() {
            println!("   (empty)");
        } else {
            for (index, user_agent) in cache.user_agent_set.iter().enumerate() {
                println!("   {:5} | {}", index, user_agent);
            }
        }
        
        // Print HashMap separately
        println!("\n3. HashMap - Index-based Key Mappings ({} entries):", cache.block_map.len());
        println!("   Numeric Key | Decoded Key | Second Field");
        println!("   ------------|-------------|-------------");
        if cache.block_map.is_empty() {
            println!("   (empty)");
        } else {
            for (key, second_field) in cache.block_map.iter() {
                let ip_index = (key >> 32) as usize;
                let ua_index = (key & 0xFFFFFFFF) as usize;
                
                if ua_index == 0 && ip_index == (*key as usize) {
                    // IP index only key
                    let invalid_ip = "INVALID".to_string();
                    let ip_addr = cache.ip_set.get_index(ip_index).unwrap_or(&invalid_ip);
                    println!("   {:11} | IP_idx: {} ({}) | {}", key, ip_index, ip_addr, second_field);
                } else {
                    // Combined key (IP index + UA index)
                    let invalid_ip = "INVALID".to_string();
                    let invalid_ua = "INVALID".to_string();
                    let ip_addr = cache.ip_set.get_index(ip_index).unwrap_or(&invalid_ip);
                    let ua_str = cache.user_agent_set.get_index(ua_index).unwrap_or(&invalid_ua);
                    println!("   {:11} | IP_idx: {} ({}), UA_idx: {} ({}) | {}", 
                             key, ip_index, ip_addr, ua_index, ua_str, second_field);
                }
            }
        }
        
        println!("\n==========================");
    }
}