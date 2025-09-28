Block Cache
===========
Imperva anti-scraping service works by blocking malicious HTTP clients. An HTTP
client always has an IP and a user agent string (that might be empty). When an
HTTP client is classified as a bot, a block is created. Blocks are produced by
the back-end and distributed to a block cache service at the edge nodes.

Blocks
------
Blocks are always based on an IP address. Blocking all clients on an IP (full IP
block) is sometimes too blunt (e.g., HTTP proxies), hence a block can optionally
be limited to a specific user agent on the specified IP (user agent block). Each
block also has a type != 0, this makes it possible to have different kinds of
blocks, such as captcha blocks.

Assessment
----------
Your task is to implement a block cache in Rust with the following public
signature:

```rust
pub struct BlockCache {}

impl BlockCache {
    /// # Parameters
    /// * block_file_name - File to load blocks from
    pub fn new(block_file_name: &str) -> Self {
        todo!()
    }

    /// Returns block type where 0 represents not blocked
    ///
    /// # Parameters
    /// * ip - Client's IPv4 address
    /// * user_agent - Client's user agent string
    pub fn get_block(&self, ip: &str, user_agent: &str) -> u32 {
        todo!()
    }
}
```

Assume that an external service is periodically producing a file with blocks,
called a block file, and moving it to `block_file_name` with `mv` (which is
atomic).

Blocks should be reloaded from the block file every 5 seconds.

get_block() needs to be responsive, i.e., low latency, even when a large block
file is being loaded. During the initial load of block file, it can return 0
(i.e., not blocked) for all HTTP clients.

HTTP clients not matching a row in the block file should not be blocked (return
0). A full IP block always takes precedence over user agent blocks.

If the assessment is underspecified, feel free to ask or make your own
assumptions and document.

Block File
----------
The provided block file consists of rows with the following format:

```
<ip>,<block_type>[,<user_agent>]
```

Using the following block file:

```
10.0.0.1,1
10.0.0.2,3
10.0.0.2,2,Python-urllib/2.6
192.168.1.1,1,Googlebot/2.1
192.168.1.1,1,curl
192.168.1.1,2,
192.168.1.1,1,Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.116 Safari/537.36 OPR/40.0.2308.81
```

get_block() should return:

```
block_cache.get_block("192.0.2.0", "Mozilla") => 0
block_cache.get_block("10.0.0.1", "Mozilla") => 1
block_cache.get_block("10.0.0.2", "Mozilla") => 3
block_cache.get_block("10.0.0.2", "Python-urllib/2.6") => 3 // Full IP block takes precedence
block_cache.get_block("192.168.1.1", "curl") => 1
block_cache.get_block("192.168.1.1", "Mozilla") => 0
block_cache.get_block("192.168.1.1", "") => 2 // Matches user agent block with empty user agent
```
