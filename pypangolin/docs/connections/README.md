# Database Connections Documentation

Secure credential management for database connections with encrypted storage.

## Overview

PyPangolin provides secure credential management for database connections using Fernet encryption. Credentials are encrypted before storage and decrypted only when creating connections.

## Main Guide

- **[Database Connections Overview](../connections.md)** - Complete guide to database connection management

## SQL Databases

- **[PostgreSQL](postgresql.md)** ✅ *Tested* - Open-source relational database
- **[MySQL](mysql.md)** ✅ *Tested* - Popular relational database  
- **[Amazon Redshift](redshift.md)** ⚠️ *Untested* - Cloud data warehouse (Postgres-compatible)

## NoSQL Databases

- **[MongoDB](mongodb.md)** ✅ *Tested* - Document database

## Cloud Data Warehouses

- **[Snowflake](snowflake.md)** ⚠️ *Untested* - Cloud data platform
- **[Azure Synapse](synapse.md)** ⚠️ *Untested* - Microsoft analytics service
- **[Google BigQuery](bigquery.md)** ⚠️ *Untested* - Serverless data warehouse

## Analytics Platforms

- **[Dremio](dremio.md)** ✅ *Tested* - Data lakehouse platform with Arrow Flight

## Quick Start

```bash
# Install with database support
pip install "pypangolin[postgres]"
pip install "pypangolin[mysql]"
pip install "pypangolin[mongodb]"
pip install "pypangolin[dremio]"

# Or install all database support
pip install "pypangolin[all-connections]"
```

## Security

All database connections use Fernet encryption for credential storage. See individual guides for security best practices and key management strategies.
