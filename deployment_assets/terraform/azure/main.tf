terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.0"
    }
  }
}

provider "azurerm" {
  features {}
}

variable "location" {
  default = "East US"
}

variable "project_name" {
  default = "pangolin"
}

variable "db_password" {
  description = "Database administrator password"
  sensitive   = true
}

resource "azurerm_resource_group" "rg" {
  name     = "${var.project_name}-resources"
  location = var.location
}

# Storage Account (Blob)
resource "azurerm_storage_account" "sa" {
  name                     = "${var.project_name}sa${random_id.suffix.hex}"
  resource_group_name      = azurerm_resource_group.rg.name
  location                 = azurerm_resource_group.rg.location
  account_tier             = "Standard"
  account_replication_type = "LRS"
}

resource "random_id" "suffix" {
  byte_length = 4
}

# Azure Database for PostgreSQL
resource "azurerm_postgresql_server" "pg" {
  name                = "${var.project_name}-pg-${random_id.suffix.hex}"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name

  sku_name = "B_Gen5_1"

  storage_mb                   = 5120
  backup_retention_days        = 7
  geo_redundant_backup_enabled = false
  auto_grow_enabled            = true

  administrator_login          = "pangolinadmin"
  administrator_login_password = var.db_password
  version                      = "11"
  ssl_enforcement_enabled      = true
}

# Firewall Rule (Open to All - Template Only)
resource "azurerm_postgresql_firewall_rule" "allow_all" {
  name                = "allow-all"
  resource_group_name = azurerm_resource_group.rg.name
  server_name         = azurerm_postgresql_server.pg.name
  start_ip_address    = "0.0.0.0"
  end_ip_address      = "255.255.255.255"
}

output "storage_account_name" {
  value = azurerm_storage_account.sa.name
}

output "storage_account_key" {
  sensitive = true
  value     = azurerm_storage_account.sa.primary_access_key
}

output "database_host" {
  value = azurerm_postgresql_server.pg.fqdn
}
