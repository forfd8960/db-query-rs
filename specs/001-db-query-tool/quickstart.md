# Quickstart Guide: Database Query Tool

**Purpose**: Get the database query tool running locally in under 5 minutes  
**Created**: 2025-12-19  
**Audience**: Developers setting up the tool for the first time

## Prerequisites

- **Rust**: 1.75 or later ([Install Rust](https://rustup.rs/))
- **PostgreSQL**: Any accessible PostgreSQL database (local or remote)
- **OpenAI API Key**: For natural language query feature ([Get API key](https://platform.openai.com/api-keys))

## Quick Start (3 Steps)

### 1. Clone and Build

```bash
# Clone the repository
git clone <repository-url>
cd db-query-rs

# Build the project
cargo build --release

# The binary will be at: target/release/db-query-rs
```

### 2. Configure Environment

```bash
# Set your OpenAI API key
export OPENAI_API_KEY="sk-..."

# Optional: Configure server port (defaults to 3000)
export PORT=8000
```

### 3. Run the Server

```bash
# Start the server
cargo run --release

# Server starts at http://localhost:3000
# SQLite metadata storage created at: ./db-query/db_query.db
```

**Server Ready!** You should see:
```
🚀 Database Query Tool running on http://localhost:3000
📁 Metadata storage: ./db-query/db_query.db
✅ CORS enabled for all origins
```

---

## First Query (2 Minutes)

### Step 1: Add a Database Connection

```bash
curl -X POST http://localhost:3000/api/v1/databases \
  -H "Content-Type: application/json" \
  -d '{
    "url": "postgresql://user:password@localhost:5432/mydb"
  }'
```

**Response**:
```json
{
  "id": 1,
  "name": "mydb",
  "connectionUrl": "postgresql://user:***@localhost:5432/mydb",
  "createdAt": "2025-12-19T10:30:00Z",
  "updatedAt": "2025-12-19T10:30:00Z"
}
```

### Step 2: Retrieve Database Metadata

```bash
curl http://localhost:3000/api/v1/databases/mydb/metadata
```

**Response** (cached for fast subsequent access):
```json
{
  "tables": [
    {
      "schemaName": "public",
      "tableName": "users",
      "tableType": "TABLE",
      "columns": [
        {
          "columnName": "id",
          "dataType": "integer",
          "isNullable": false,
          "isPrimaryKey": true
        },
        {
          "columnName": "username",
          "dataType": "varchar",
          "isNullable": false,
          "isPrimaryKey": false
        }
      ]
    }
  ]
}
```

### Step 3: Execute a SQL Query

```bash
curl -X POST http://localhost:3000/api/v1/databases/mydb/query \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT id, username FROM users LIMIT 5"
  }'
```

**Response**:
```json
{
  "columns": ["id", "username"],
  "rows": [
    {"id": 1, "username": "alice"},
    {"id": 2, "username": "bob"}
  ],
  "rowCount": 2,
  "executionTime": 12.5
}
```

### Step 4: Try Natural Language Query (Bonus!)

```bash
curl -X POST http://localhost:3000/api/v1/databases/mydb/nl-query \
  -H "Content-Type: application/json" \
  -d '{
    "nlQuery": "Show me the 10 most recent users"
  }'
```

**Response**:
```json
{
  "nlQuery": "Show me the 10 most recent users",
  "generatedSql": "SELECT * FROM users ORDER BY created_at DESC LIMIT 10",
  "result": {
    "columns": ["id", "username", "email", "createdAt"],
    "rows": [ /* ... */ ],
    "rowCount": 10
  }
}
```

---

## Common Operations

### List All Databases

```bash
curl http://localhost:3000/api/v1/databases
```

### Update Database Connection

```bash
curl -X PUT http://localhost:3000/api/v1/databases/mydb \
  -H "Content-Type: application/json" \
  -d '{
    "url": "postgresql://newuser:newpass@localhost:5432/mydb"
  }'
```

### Execute Complex Query

```bash
curl -X POST http://localhost:3000/api/v1/databases/mydb/query \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT u.username, COUNT(o.id) as order_count FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.username"
  }'
```

**Note**: Query automatically limited to 1000 rows if no LIMIT specified.

---

## Safety Features (Automatic)

### ✅ SELECT-Only Validation

```bash
# This will be REJECTED
curl -X POST http://localhost:3000/api/v1/databases/mydb/query \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "DELETE FROM users WHERE id = 1"
  }'
```

**Response** (400 Bad Request):
```json
{
  "error": "ValidationError",
  "message": "Only SELECT statements are allowed",
  "details": "Found DELETE statement at position 0"
}
```

### ✅ Automatic LIMIT Injection

```bash
# Missing LIMIT? No problem, automatically adds LIMIT 1000
curl -X POST http://localhost:3000/api/v1/databases/mydb/query \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELECT * FROM users"
  }'
```

**Executed as**: `SELECT * FROM users LIMIT 1000`

### ✅ SQL Syntax Validation

```bash
# Malformed SQL is caught before execution
curl -X POST http://localhost:3000/api/v1/databases/mydb/query \
  -H "Content-Type: application/json" \
  -d '{
    "sql": "SELCT * FORM users"
  }'
```

**Response** (400 Bad Request):
```json
{
  "error": "SyntaxError",
  "message": "Invalid SQL syntax",
  "details": "Expected SELECT, found SELCT at position 0"
}
```

---

## Troubleshooting

### Server Won't Start

**Problem**: `OPENAI_API_KEY not found`

**Solution**:
```bash
export OPENAI_API_KEY="sk-your-key-here"
```

**Problem**: `Address already in use (port 3000)`

**Solution**:
```bash
# Use a different port
export PORT=8080
cargo run --release
```

### Database Connection Fails

**Problem**: `Connection refused` when adding database

**Solution**:
- Verify PostgreSQL is running: `pg_isready`
- Check connection URL format: `postgresql://user:pass@host:port/dbname`
- Ensure network access to database host
- Verify credentials are correct

### Natural Language Queries Fail

**Problem**: `503 Service Unavailable` on `/nl-query`

**Solution**:
- Verify OpenAI API key is set and valid
- Check API quota/billing at [OpenAI Dashboard](https://platform.openai.com/usage)
- Test API key directly: `curl https://api.openai.com/v1/models -H "Authorization: Bearer $OPENAI_API_KEY"`

**Problem**: Generated SQL is incorrect or unsafe

**Solution**:
- This is expected for complex queries - LLM is 80% accurate
- Fallback to manual SQL entry via `/query` endpoint
- Verify database metadata is accurate (re-fetch if schema changed)

### Metadata Not Updating

**Problem**: Schema changes not reflected in metadata API

**Solution**:
- Current implementation caches indefinitely
- Workaround: Delete SQLite cache and restart
```bash
rm -rf db-query
cargo run --release
# Re-fetch metadata via GET /api/v1/databases/{name}/metadata
```

---

## Next Steps

### Development Mode

```bash
# Run with hot reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Production Deployment

```bash
# Build optimized binary
cargo build --release --target x86_64-unknown-linux-gnu

# Copy binary to server
scp target/release/db-query-rs user@server:/usr/local/bin/

# Run as systemd service (example)
# Create /etc/systemd/system/db-query.service
# Set Environment=OPENAI_API_KEY=...
# systemctl enable db-query
# systemctl start db-query
```

### API Documentation

- **OpenAPI Spec**: See [contracts/openapi.yaml](contracts/openapi.yaml)
- **Interactive Docs**: Use Swagger UI or Postman to import OpenAPI spec
- **Examples**: See [data-model.md](data-model.md) for request/response examples

### Security Considerations

⚠️ **Important**: This tool has **NO AUTHENTICATION** by design.

- **Recommended**: Deploy on trusted network only (localhost, internal network, VPN)
- **Not Recommended**: Expose to public internet without additional security layer
- **Database Credentials**: Stored in plaintext in SQLite - anyone with file access can read them
- **Network Security**: Implement firewall rules or reverse proxy authentication if needed

---

## Summary

**You're Ready to Use the Database Query Tool!**

✅ Server running on http://localhost:3000  
✅ Database connections configured  
✅ Metadata cached for fast access  
✅ SQL queries validated for safety  
✅ Natural language queries available  

For full API reference, see [contracts/openapi.yaml](contracts/openapi.yaml).

For implementation details, see [plan.md](plan.md) and [data-model.md](data-model.md).
