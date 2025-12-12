# Getting Started

## Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (version 1.92 or later)
- [Docker](https://docs.docker.com/get-docker/) (optional, for running dependencies)

## Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-repo/pangolin.git
   cd pangolin
   ```

2. **Run the Server**
   ```bash
   cd pangolin
   cargo run --bin pangolin_api
   ```
   The server will start at `http://localhost:8080`.

## Basic Usage

### 1. Create a Branch
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"name": "dev", "branch_type": "experimental"}' \
  http://localhost:8080/api/v1/branches
```

### 2. Create a Namespace
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"namespace": ["data_team"]}' \
  http://localhost:8080/v1/prefix/namespaces
```

### 3. Create a Table on the Dev Branch
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"name": "users@dev", "location": "s3://bucket/users"}' \
  http://localhost:8080/v1/prefix/namespaces/data_team/tables
```

### 4. Verify Table Exists on Dev
```bash
curl http://localhost:8080/v1/prefix/namespaces/data_team@dev/tables
```
