terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
  }
}

provider "google" {
  project = var.project_id
  region  = var.region
}

variable "project_id" {
  description = "GCP Project ID"
}

variable "region" {
  default = "us-central1"
}

variable "db_password" {
  description = "Database password"
  sensitive   = true
}

resource "random_id" "suffix" {
  byte_length = 4
}

# GCS Bucket
resource "google_storage_bucket" "lakehouse_bucket" {
  name          = "pangolin-data-${random_id.suffix.hex}"
  location      = "US"
  force_destroy = true # Convenient for demos
}

# Cloud SQL (Postgres)
resource "google_sql_database_instance" "master" {
  name             = "pangolin-sql-${random_id.suffix.hex}"
  database_version = "POSTGRES_15"
  region           = var.region

  settings {
    tier = "db-f1-micro"
    ip_configuration {
      ipv4_enabled = true # Public IP for ease of access (Template Only)
      authorized_networks {
        name  = "allow-all"
        value = "0.0.0.0/0"
      }
    }
  }
  deletion_protection = false # Convenient for demos
}

resource "google_sql_user" "users" {
  name     = "pangolin_user"
  instance = google_sql_database_instance.master.name
  password = var.db_password
}

resource "google_sql_database" "database" {
  name     = "pangolin"
  instance = google_sql_database_instance.master.name
}

output "bucket_name" {
  value = google_storage_bucket.lakehouse_bucket.name
}

output "connection_name" {
  value = google_sql_database_instance.master.connection_name
}

output "public_ip" {
  value = google_sql_database_instance.master.public_ip_address
}
