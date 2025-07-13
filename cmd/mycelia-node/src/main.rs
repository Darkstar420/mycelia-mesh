use mycelia_node::{Role, run};

#[tokio::main]
async fn main() {
    run(Role::Shim, 11434).await;
}
