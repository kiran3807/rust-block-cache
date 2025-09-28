use rust_block_cache::{constants, BlockCache};
use std::thread;
use std::time::Duration;

fn main() {
    let cache = BlockCache::new(constants::BLOCK_FILE);
    
    // Wait a moment for initial load
    thread::sleep(Duration::from_secs(1));

    println!("Initial cache contents:");
    cache.print_lru_cache_contents();

    println!("-----Testing get_block functionality-----\n");
    
    let test_cases = vec![
        ("10.0.0.1", "", "Single IP entry"),
        ("10.0.0.1", "Googlebot/2.1", "Combined IP + User Agent"),
        ("10.0.0.1", "curl", "Combined IP + User Agent"),
        ("10.0.0.2", "Python-urllib/2.6", "Combined IP + User Agent"),
        ("192.168.1.1", "Googlebot/2.1", "Combined IP + User Agent"),
        ("192.168.1.1", "curl", "Combined IP + User Agent"),
        ("999.999.999.999", "", "Non-existent IP"),
        ("10.0.0.1", "NonExistentUserAgent", "Existing IP but non-existent User Agent"),
        ("192.168.1.1", "", "IP exists but no single IP entry in hashmap"),
    ];
    
    for (ip, user_agent, description) in test_cases {
        println!("Testing: {} - IP: '{}', UA: '{}'", description, ip, user_agent);
        
        match cache.get_block(ip, user_agent) {
            Ok(value) => {
                if value == "0" {
                    println!("  Result: Not found (returned '0')");
                } else {
                    println!("  Result: Found value = '{}'", value);
                }
            }
            Err(error) => {
                println!("  Result: ERROR - {}", error);
            }
        }
        println!();
    }
    
    print!("Cache contents after get_block tests:");
    cache.print_lru_cache_contents();
    println!("----Test completed-----\n");

}