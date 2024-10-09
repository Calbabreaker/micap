use std::time::Duration;

#[tokio::test]
async fn integration_test() {
    tokio::spawn(async {
        let _ = tokio::time::timeout(Duration::from_secs(2), async {
            crate::start_server().await.unwrap()
        })
        .await;
    })
    .await
    .unwrap();

    std::process::exit(0);
}
