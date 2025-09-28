use rust_block_cache::{BlockCache};
use std::thread;
use std::time::Duration;

fn main() {
    let cache = BlockCache::new("sample_block_file.txt");
    
    println!("Press Ctrl+C to exit...");
    
    // Keep program running until user interrupts
    loop {
        thread::sleep(Duration::from_secs(7));
        cache.print_cache_contents();
    }
}