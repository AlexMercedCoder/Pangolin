#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_handlers;
    use crate::iceberg_handlers;
    use crate::system_config_handlers;
    use crate::federated_catalog_handlers;
    use crate::pangolin_handlers;
    use pangolin_store::memory::MemoryStore;
    use std::sync::Arc;
    use uuid::Uuid;
    use axum::ext_trait::RequestExt;
    use axum::extract::Request;
    use tower::ServiceExt;

    // Helper to setup app
    async fn setup() -> (axum::Router, Arc<MemoryStore>) {
        let store = Arc::new(MemoryStore::new());
        let app = crate::app(store.clone());
        (app, store)
    }

    #[tokio::test]
    async fn test_system_settings() {
        let (app, store) = setup().await;
        // Test logic would go here, simulating HTTP requests
        // verifying store state
    }
}
