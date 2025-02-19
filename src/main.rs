use bluetooth::windows::winrt;


mod bluetooth;
mod constant;
mod protocols;

#[tokio::main]
async fn main() { 
    winrt::shit().await.expect("bruh");
}
