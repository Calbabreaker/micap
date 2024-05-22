#[tokio::main]
async fn main() {
    mycap_server::setup_log();
    mycap_server::start_server().await;
}
