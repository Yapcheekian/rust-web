use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    hc.do_get("/index.html").await?.print().await?;

    hc.do_get("/hello2/ppppp").await?.print().await?;

    hc.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "pwd": "welcome"
        }),
    )
    .await?
    .print()
    .await?;

    hc.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "create_task",
            "params": {
                "data": {
                    "title": "task1",
                }
            }
        }),
    )
    .await?
    .print()
    .await?;

    hc.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "list_tasks",
        }),
    )
    .await?
    .print()
    .await?;

    Ok(())
}
