
#[tokio::main]
async fn main() {
    println!("Hello, async world!");
    
    // Example async operation
    let result = async_task().await;
    println!("Result: {}", result);

}


async fn async_task() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
    "Task completed".to_string()
}