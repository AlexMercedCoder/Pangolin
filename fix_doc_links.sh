#!/bin/bash
# Comprehensive documentation link fix script

# Fix docs/README.md - replace storage/ with warehouse/
sed -i 's|storage/|warehouse/|g' docs/README.md
sed -i 's|research/|../planning/research/|g' docs/README.md

# Fix docs/api/README.md
sed -i 's|../storage/|../warehouse/|g' docs/api/README.md

# Fix docs/api/authentication.md
sed -i 's|security_vending.md|../features/security_vending.md|g' docs/api/authentication.md

# Fix docs/getting-started/README.md
sed -i 's|../storage/|../warehouse/|g' docs/getting-started/README.md

# Fix docs/getting-started/getting_started.md
sed -i 's|branch_management.md|../features/branch_management.md|g' docs/getting-started/getting_started.md
sed -i 's|storage_s3.md|../warehouse/s3.md|g' docs/getting-started/getting_started.md
sed -i 's|warehouse_management.md|../features/warehouse_management.md|g' docs/getting-started/getting_started.md
sed -i 's|\./pyiceberg_testing.md|../features/pyiceberg_testing.md|g' docs/getting-started/getting_started.md
sed -i 's|\./warehouse_management.md|../features/warehouse_management.md|g' docs/getting-started/getting_started.md
sed -i 's|\./api/|../api/|g' docs/getting-started/getting_started.md

# Fix docs/features/README.md
sed -i 's|../storage/|../warehouse/|g' docs/features/README.md

# Fix docs/features/warehouse_management.md
sed -i 's|../storage/storage_s3.md|../warehouse/s3.md|g' docs/features/warehouse_management.md

# Fix docs/features/iam_roles.md
sed -i 's|../storage/storage_s3.md|../warehouse/s3.md|g' docs/features/iam_roles.md
sed -i 's|../storage/storage_azure.md|../warehouse/azure.md|g' docs/features/iam_roles.md
sed -i 's|../storage/storage_gcs.md|../warehouse/gcs.md|g' docs/features/iam_roles.md

# Fix docs/features/security_vending.md
sed -i 's|../storage/storage_s3.md|../warehouse/s3.md|g' docs/features/security_vending.md

# Fix docs/features/pyiceberg_testing.md
sed -i 's|\./client_configuration.md|../getting-started/client_configuration.md|g' docs/features/pyiceberg_testing.md
sed -i 's|\./getting_started.md|../getting-started/getting_started.md|g' docs/features/pyiceberg_testing.md

# Fix docs/federated_catalogs.md
sed -i 's|\.\./permissions.md|./permissions.md|g' docs/federated_catalogs.md

# Fix docs/service_users.md
sed -i 's|\./rbac.md|./features/rbac.md|g' docs/service_users.md
sed -i 's|\./api_reference.md|./api/api_overview.md|g' docs/service_users.md

# Fix docs/warehouse/README.md - remove anchor links that don't work
sed -i 's|s3.md#client-configuration|s3.md|g' docs/warehouse/README.md
sed -i 's|azure.md#client-configuration|azure.md|g' docs/warehouse/README.md
sed -i 's|gcs.md#client-configuration|gcs.md|g' docs/warehouse/README.md

echo "Documentation links fixed!"
