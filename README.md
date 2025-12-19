# Database Query Tool (db-query-rs)

A PostgreSQL database exploration and query tool with natural language query support.

## Features

- 🔌 **Database Connection Management** - Add, list, and update PostgreSQL connections
- 📊 **Schema Metadata Browsing** - Explore tables, views, and columns with intelligent caching
- 🔒 **Safe SQL Execution** - Automatic validation (SELECT-only) and LIMIT injection
- 🤖 **Natural Language Queries** - Translate natural language to SQL using OpenAI

## Quick Start

### Prerequisites

- **Rust 1.75+** ([Install Rust](https://rustup.rs/))
- **PostgreSQL** database (local or remote)
- **OpenAI API Key** ([Get API key](https://platform.openai.com/api-keys))

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd db-query-rs

# Copy environment template
cp .env.example .env

# Edit .env and add your OpenAI API key
# OPENAI_API_KEY=sk-your-key-here

# Build and run
cargo run --release
```

Server starts at **http://localhost:3000**

### First Query

```bash
# 1. Add a database connection
curl -X POST http://localhost:3000/api/v1/databases \
  -H "Content-Type: application/json" \
  -d '{"url": "postgresql://user:password@localhost:5432/mydb"}'

# 2. Get database metadata
curl http://localhost:3000/api/v1/databases/mydb/metadata

# 3. Execute a SQL query
curl -X POST http://localhost:3000/api/v1/databases/mydb/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT id, username FROM users LIMIT 5"}'

# 4. Try natural language (requires OPENAI_API_KEY)
curl -X POST http://localhost:3000/api/v1/databases/mydb/nl-query \
  -H "Content-Type: application/json" \
  -d '{"nlQuery": "Show me the 10 most recent users"}'
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/databases` | Add database connection |
| GET | `/api/v1/databases` | List all connections |
| PUT | `/api/v1/databases/{name}` | Update connection |
| GET | `/api/v1/databases/{name}/metadata` | Get schema metadata |
| POST | `/api/v1/databases/{name}/query` | Execute SQL query |
| POST | `/api/v1/databases/{name}/nl-query` | Natural language query |

## Safety Features

✅ **SELECT-Only Validation** - Only SELECT statements allowed, all DML/DDL rejected  
✅ **Automatic LIMIT** - Queries without LIMIT get `LIMIT 1000` appended  
✅ **SQL Syntax Validation** - Malformed SQL caught before execution  
✅ **camelCase Responses** - All API responses use camelCase field names  
✅ **CORS Enabled** - Accessible from any origin

## Architecture

```
src/
├── models/        # Data entities (DatabaseConnection, TableMetadata, etc.)
├── services/      # Business logic (database, metadata, query, LLM)
├── api/           # HTTP handlers and routes
├── storage/       # SQLite repository for caching
└── validation/    # SQL validation and parsing
```

## Configuration

Environment variables:

- `OPENAI_API_KEY` - OpenAI API key (required for natural language queries)
- `PORT` - Server port (default: 3000)
- `RUST_LOG` - Log level (default: info)

## Development

```bash
# Run in development mode
cargo run

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Build optimized release binary
cargo build --release
```

## Documentation

- **Specification**: [specs/001-db-query-tool/spec.md](specs/001-db-query-tool/spec.md)
- **Implementation Plan**: [specs/001-db-query-tool/plan.md](specs/001-db-query-tool/plan.md)
- **API Contracts**: [specs/001-db-query-tool/contracts/openapi.yaml](specs/001-db-query-tool/contracts/openapi.yaml)
- **Quickstart Guide**: [specs/001-db-query-tool/quickstart.md](specs/001-db-query-tool/quickstart.md)

## Security Considerations

⚠️ **No Authentication** - This tool has no built-in authentication by design.

- Deploy on **trusted networks only** (localhost, VPN, internal network)
- Database credentials stored in **plaintext** in SQLite
- **Do not expose** to public internet without additional security layer

## License

MIT
