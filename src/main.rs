
use rust_block_cache::{BlockCache};

fn main() {
    println!("Hello, async world!");

    // Example usage of BlockCache
    let cache = BlockCache::new("test.dat");
    let block = cache.get_block("127.0.0.1", "Mozilla/5.0");
    println!("Block: {}", block);
}