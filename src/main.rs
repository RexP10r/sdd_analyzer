/// @req SCS-CLI-001
#[tokio::main]
async fn main() {
    let state = sdd_navigator::state::new_app_state();
    if let Err(e) = sdd_navigator::server::run(state).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}
