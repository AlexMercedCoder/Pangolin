// Regression tests for CLI optimization commands
// Tests the 5 new commands: stats, catalog-summary, search, bulk-delete, validate

#[cfg(test)]
mod optimization_command_tests {
    use pangolin_cli_common::optimization_types::*;
    use serde_json;

    #[test]
    fn test_dashboard_stats_deserialization() {
        let json = r#"{
            "catalogs_count": 5,
            "tables_count": 10,
            "namespaces_count": 3,
            "users_count": 2,
            "warehouses_count": 1,
            "branches_count": 4,
            "scope": "tenant"
        }"#;

        let stats: DashboardStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.catalogs_count, 5);
        assert_eq!(stats.tables_count, 10);
        assert_eq!(stats.scope, "tenant");
    }

    #[test]
    fn test_catalog_summary_deserialization() {
        let json = r#"{
            "name": "test_catalog",
            "table_count": 15,
            "namespace_count": 5,
            "branch_count": 2,
            "storage_location": "s3://bucket/path"
        }"#;

        let summary: CatalogSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.name, "test_catalog");
        assert_eq!(summary.table_count, 15);
        assert_eq!(summary.storage_location, Some("s3://bucket/path".to_string()));
    }

    #[test]
    fn test_catalog_summary_without_storage() {
        let json = r#"{
            "name": "test_catalog",
            "table_count": 0,
            "namespace_count": 0,
            "branch_count": 0,
            "storage_location": null
        }"#;

        let summary: CatalogSummary = serde_json::from_str(json).unwrap();
        assert_eq!(summary.storage_location, None);
    }

    #[test]
    fn test_search_response_deserialization() {
        let json = r#"{
            "results": [
                {
                    "id": "123e4567-e89b-12d3-a456-426614174000",
                    "name": "test_table",
                    "namespace": ["default"],
                    "catalog": "test_catalog",
                    "asset_type": "table"
                }
            ],
            "total": 1,
            "limit": 50,
            "offset": 0
        }"#;

        let response: SearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total, 1);
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].name, "test_table");
        assert_eq!(response.results[0].namespace, vec!["default"]);
    }

    #[test]
    fn test_search_response_empty() {
        let json = r#"{
            "results": [],
            "total": 0,
            "limit": 50,
            "offset": 0
        }"#;

        let response: SearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total, 0);
        assert_eq!(response.results.len(), 0);
    }

    #[test]
    fn test_bulk_operation_response_success() {
        let json = r#"{
            "succeeded": 5,
            "failed": 0,
            "errors": []
        }"#;

        let response: BulkOperationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.succeeded, 5);
        assert_eq!(response.failed, 0);
        assert!(response.errors.is_empty());
    }

    #[test]
    fn test_bulk_operation_response_with_errors() {
        let json = r#"{
            "succeeded": 3,
            "failed": 2,
            "errors": ["Asset not found: uuid1", "Permission denied: uuid2"]
        }"#;

        let response: BulkOperationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.succeeded, 3);
        assert_eq!(response.failed, 2);
        assert_eq!(response.errors.len(), 2);
        assert!(response.errors[0].contains("not found"));
    }

    #[test]
    fn test_validate_names_response() {
        let json = r#"{
            "results": [
                {
                    "name": "available_catalog",
                    "available": true,
                    "reason": null
                },
                {
                    "name": "taken_catalog",
                    "available": false,
                    "reason": "Name already exists"
                }
            ]
        }"#;

        let response: ValidateNamesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 2);
        assert_eq!(response.results[0].available, true);
        assert_eq!(response.results[1].available, false);
        assert_eq!(response.results[1].reason, Some("Name already exists".to_string()));
    }

    #[test]
    fn test_validate_names_all_available() {
        let json = r#"{
            "results": [
                {"name": "catalog1", "available": true, "reason": null},
                {"name": "catalog2", "available": true, "reason": null},
                {"name": "catalog3", "available": true, "reason": null}
            ]
        }"#;

        let response: ValidateNamesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.len(), 3);
        assert!(response.results.iter().all(|r| r.available));
    }

    #[test]
    fn test_asset_search_result_nested_namespace() {
        let json = r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "sales_data",
            "namespace": ["analytics", "sales", "monthly"],
            "catalog": "production",
            "asset_type": "table"
        }"#;

        let result: AssetSearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.namespace.len(), 3);
        assert_eq!(result.namespace.join("."), "analytics.sales.monthly");
    }
}
