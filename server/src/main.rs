#[tokio::main]
async fn main() -> anyhow::Result<()> {
    micap_server::setup_log();
    micap_server::start_server().await
}
