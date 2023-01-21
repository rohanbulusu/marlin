mod marlin;

#[tokio::main]
async fn main() {
    marlin::run().await;
}
