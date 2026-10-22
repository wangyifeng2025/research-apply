#[tokio::main]
async fn main() {
    if let Err(error) = research_apply_server::seed::run().await {
        eprintln!("种子数据写入失败: {error}");
        std::process::exit(1);
    }
}
