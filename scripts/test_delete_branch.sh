#!/bin/bash
set -e

echo "🧪 Testing delete_branch with cascade delete across all backends..."

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

# Test function for each backend
test_backend() {
    local backend=$1
    local db_url=$2
    
    echo -e "${BLUE}Testing $backend backend...${NC}"
    
    # Create test database/file
    case $backend in
        "postgres")
            docker-compose -f docker-compose.db-test.yml up -d postgres
            sleep 5
            ;;
        "sqlite")
            rm -f /tmp/test_delete_branch.db
            ;;
        "mongo")
            docker-compose -f docker-compose.db-test.yml up -d mongo
            sleep 5
            ;;
    esac
    
    # Run Rust test
    cd pangolin/pangolin_store
    
    cat > /tmp/test_delete_branch_${backend}.rs << 'EOF'
use pangolin_store::*;
use pangolin_core::model::{Branch, BranchType, Asset, AssetType};
use uuid::Uuid;

#[tokio::test]
async fn test_delete_branch_cascade() {
    let tenant_id = Uuid::new_v4();
    let catalog_name = "test_catalog";
    let branch_name = "test_branch";
    
    // Create store based on backend
    let store: Box<dyn CatalogStore> = match std::env::var("TEST_BACKEND").unwrap().as_str() {
        "postgres" => {
            let url = std::env::var("DATABASE_URL").unwrap();
            Box::new(PostgresStore::new(&url).await.unwrap())
        },
        "sqlite" => {
            let url = std::env::var("DATABASE_URL").unwrap();
            let store = SqliteStore::new(&url).await.unwrap();
            let schema = include_str!("../../sql/sqlite_schema.sql");
            store.apply_schema(schema).await.unwrap();
            Box::new(store)
        },
        "mongo" => {
            let url = std::env::var("DATABASE_URL").unwrap();
            let db_name = std::env::var("MONGO_DB_NAME").unwrap_or("test".to_string());
            Box::new(MongoStore::new(&url, &db_name).await.unwrap())
        },
        _ => panic!("Unknown backend")
    };
    
    // Create branch
    let branch = Branch {
        name: branch_name.to_string(),
        head_commit_id: None,
        branch_type: BranchType::Experimental,
        assets: vec![],
    };
    store.create_branch(tenant_id, catalog_name, branch).await.unwrap();
    
    // Create assets in the branch
    for i in 0..3 {
        let asset = Asset {
            id: Uuid::new_v4(),
            name: format!("test_asset_{}", i),
            asset_type: AssetType::IcebergTable,
            metadata_location: Some(format!("s3://bucket/table_{}", i)),
            schema: None,
            properties: std::collections::HashMap::new(),
        };
        store.create_asset(
            tenant_id,
            catalog_name,
            Some(branch_name.to_string()),
            vec!["test_ns".to_string()],
            asset
        ).await.unwrap();
    }
    
    // Verify assets exist
    let assets_before = store.list_assets(
        tenant_id,
        catalog_name,
        Some(branch_name.to_string()),
        vec!["test_ns".to_string()]
    ).await.unwrap();
    assert_eq!(assets_before.len(), 3, "Should have 3 assets before delete");
    
    // Delete branch
    store.delete_branch(tenant_id, catalog_name, branch_name.to_string()).await.unwrap();
    
    // Verify branch is deleted
    let branch_result = store.get_branch(tenant_id, catalog_name, branch_name.to_string()).await.unwrap();
    assert!(branch_result.is_none(), "Branch should be deleted");
    
    // Verify assets are cascade deleted
    let assets_after = store.list_assets(
        tenant_id,
        catalog_name,
        Some(branch_name.to_string()),
        vec!["test_ns".to_string()]
    ).await.unwrap();
    assert_eq!(assets_after.len(), 0, "Assets should be cascade deleted");
    
    println!("✅ {} backend: delete_branch with cascade delete works!", std::env::var("TEST_BACKEND").unwrap());
}
EOF
    
    # Run test
    export TEST_BACKEND=$backend
    export DATABASE_URL=$db_url
    export MONGO_DB_NAME="test_delete_branch"
    
    cargo test --test delete_branch_test -- --nocapture || {
        echo -e "${RED}❌ $backend test failed${NC}"
        cd ../..
        return 1
    }
    
    cd ../..
    echo -e "${GREEN}✅ $backend test passed${NC}"
    
    # Cleanup
    case $backend in
        "postgres")
            docker-compose -f docker-compose.db-test.yml down postgres
            ;;
        "sqlite")
            rm -f /tmp/test_delete_branch.db
            ;;
        "mongo")
            docker-compose -f docker-compose.db-test.yml down mongo
            ;;
    esac
}

# Test all backends
echo "Testing Postgres..."
test_backend "postgres" "postgresql://testuser:testpass@localhost:5432/testdb"

echo ""
echo "Testing SQLite..."
test_backend "sqlite" "sqlite:///tmp/test_delete_branch.db"

echo ""
echo "Testing MongoDB..."
test_backend "mongo" "mongodb://testuser:testpass@localhost:27017"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All backends pass delete_branch cascade delete test!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
