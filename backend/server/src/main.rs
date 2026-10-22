#[tokio::main]
async fn main() -> Result<(), research_apply_server::error::AppError> {
    research_apply_server::run().await
}
