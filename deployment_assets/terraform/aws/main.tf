terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

variable "aws_region" {
  default = "us-east-1"
}

variable "project_name" {
  default = "pangolin"
}

variable "db_password" {
  description = "Database password"
  sensitive   = true
}

# S3 Bucket for Iceberg Data
resource "aws_s3_bucket" "lakehouse_bucket" {
  bucket_prefix = "${var.project_name}-data-"
}

# RDS Postgres Instance
resource "aws_db_instance" "pangolin_metadata" {
  allocated_storage    = 20
  db_name              = "pangolin"
  engine               = "postgres"
  engine_version       = "15"
  instance_class       = "db.t3.micro"
  username             = "pangolin_user"
  password             = var.db_password
  skip_final_snapshot  = true
  publicly_accessible  = true # WARNING: For demo/ease of use only. Secure this in prod.
}

output "s3_bucket_name" {
  value = aws_s3_bucket.lakehouse_bucket.id
}

output "database_endpoint" {
  value = aws_db_instance.pangolin_metadata.endpoint
}

output "database_url" {
  sensitive = true
  value = "postgresql://pangolin_user:${var.db_password}@${aws_db_instance.pangolin_metadata.endpoint}/pangolin"
}
