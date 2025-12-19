# Research: Database Query Tool

**Purpose**: Resolve technical clarifications and research best practices for implementation  
**Created**: 2025-12-19  
**Status**: Complete

## Research Tasks

### 1. SQL Parser Library Selection (NEEDS CLARIFICATION from Technical Context)

**Task**: Identify the best Rust SQL parser library for validating SELECT-only queries and detecting LIMIT clauses.

**Decision**: Use **sqlparser-rs** (sqlparser crate)

**Rationale**:
- Most mature and widely-used SQL parser in Rust ecosystem (10k+ downloads/day)
- Supports PostgreSQL dialect specifically
- Can parse SQL into AST, allowing inspection of statement types (SELECT vs DML/DDL)
- Enables detection of LIMIT clause presence in parsed queries
- Actively maintained with good PostgreSQL compatibility
- Zero-copy parsing for performance

**Alternatives Considered**:
- **pg_query.rs**: Bindings to libpg_query (actual PostgreSQL parser)
  - Rejected: Requires C dependencies, more complex to build/deploy
  - Advantage: 100% PostgreSQL-compatible parsing
  - Disadvantage: Heavier dependency, less ergonomic Rust API
- **sql-parse**: Simpler parser
  - Rejected: Less feature-complete, limited dialect support
  - Disadvantage: May not handle all PostgreSQL-specific syntax
- **Custom regex/string matching**:
  - Rejected: Unsafe, easily bypassed, unreliable for complex queries
  - Cannot handle comments, nested queries, or complex syntax

**Implementation Notes**:
- Parse query with `Parser::parse_sql(&PostgreSqlDialect{}, sql)`
- Check AST root is `Statement::Query` (reject if `Statement::Insert/Update/Delete/Drop`)
- Traverse query AST to detect `LIMIT` clause
- If no LIMIT found, append " LIMIT 1000" to original SQL string before execution

### 2. PostgreSQL Metadata Retrieval Best Practices

**Task**: Research optimal approach for querying PostgreSQL system catalogs to retrieve table/view/column metadata.

**Decision**: Use **information_schema** queries with selective filtering

**Rationale**:
- `information_schema` is ANSI SQL standard, portable across PostgreSQL versions
- Provides structured views for tables, columns, constraints
- Well-documented and stable API
- SQLx supports querying information_schema directly

**Queries**:
```sql
-- Tables and views
SELECT 
  table_schema, 
  table_name, 
  table_type 
FROM information_schema.tables 
WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
ORDER BY table_schema, table_name;

-- Columns
SELECT 
  table_schema,
  table_name,
  column_name,
  data_type,
  is_nullable,
  column_default
FROM information_schema.columns
WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
ORDER BY table_schema, table_name, ordinal_position;

-- Constraints (primary keys, foreign keys)
SELECT
  tc.table_schema,
  tc.table_name,
  tc.constraint_type,
  kcu.column_name
FROM information_schema.table_constraints tc
JOIN information_schema.key_column_usage kcu
  ON tc.constraint_name = kcu.constraint_name
WHERE tc.table_schema NOT IN ('pg_catalog', 'information_schema')
ORDER BY tc.table_schema, tc.table_name, kcu.ordinal_position;
```

**Alternatives Considered**:
- **pg_catalog system tables** (pg_class, pg_attribute, etc.):
  - Advantage: More complete information, PostgreSQL-specific details
  - Rejected: Requires knowledge of PostgreSQL internals, more complex joins
  - Use case: Only if information_schema proves insufficient
- **LLM-based schema extraction**:
  - Rejected: Unnecessary LLM call for structured data
  - LLM should only be used for natural language → SQL, not schema extraction
  - Schema is already in structured format in database

### 3. LLM Prompt Engineering for SQL Generation

**Task**: Research best practices for constructing LLM prompts that generate accurate, safe SQL from natural language.

**Decision**: Use **structured prompt with schema context and examples**

**Prompt Template**:
```
You are a PostgreSQL SQL query generator. Generate ONLY a SELECT statement based on the user's request.

Database Schema:
{table and column metadata in structured format}

Rules:
1. Generate ONLY SELECT statements (no INSERT, UPDATE, DELETE, DROP)
2. Use proper PostgreSQL syntax
3. Reference only tables and columns that exist in the schema
4. Include appropriate WHERE clauses for filtering
5. Add ORDER BY if sorting is mentioned
6. Return ONLY the SQL query, no explanations

User Request: {natural_language_query}

SQL Query:
```

**Rationale**:
- Explicit constraint: SELECT-only in prompt reduces generation of unsafe queries
- Schema context helps LLM choose correct table/column names
- Clear rules improve consistency
- "Return ONLY SQL" reduces parsing complexity on response
- Schema prevents hallucination of non-existent tables/columns

**Best Practices**:
- Include all relevant table/column names in context
- Limit schema context to relevant tables if metadata is large (token limits)
- Strip markdown code fences from response (```sql...```)
- Validate generated SQL with same sqlparser before execution
- Set temperature=0 for deterministic, conservative SQL generation

**Alternatives Considered**:
- **Function calling / structured output**:
  - OpenAI function calling could enforce SELECT-only via schema
  - Advantage: More reliable structure
  - Rejected: Requires specific OpenAI API features, less portable
  - May revisit if prompt-based approach proves unreliable
- **Fine-tuned model**:
  - Rejected: Overkill for this use case, requires training data
  - GPT-4 with good prompts is sufficient for SQL generation

### 4. SQLite Schema Design for Metadata Caching

**Task**: Design optimal SQLite schema for storing database connections and cached metadata.

**Decision**: Relational schema with three tables

**Schema**:
```sql
CREATE TABLE database_connections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    connection_url TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE table_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    database_id INTEGER NOT NULL,
    schema_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    table_type TEXT NOT NULL, -- 'TABLE' or 'VIEW'
    cached_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (database_id) REFERENCES database_connections(id) ON DELETE CASCADE,
    UNIQUE (database_id, schema_name, table_name)
);

CREATE TABLE column_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_id INTEGER NOT NULL,
    column_name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    is_nullable BOOLEAN NOT NULL,
    column_default TEXT,
    ordinal_position INTEGER NOT NULL,
    is_primary_key BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (table_id) REFERENCES table_metadata(id) ON DELETE CASCADE,
    UNIQUE (table_id, column_name)
);

CREATE INDEX idx_table_metadata_db ON table_metadata(database_id);
CREATE INDEX idx_column_metadata_table ON column_metadata(table_id);
```

**Rationale**:
- Normalized design: separate tables for connections, tables, columns
- CASCADE deletes: removing database removes all cached metadata
- Timestamps: track cache freshness (can implement TTL invalidation later)
- Indexes: optimize metadata retrieval joins
- UNIQUE constraints: prevent duplicate entries

**Cache Invalidation Strategy**:
- Initial implementation: cache indefinitely (metadata rarely changes)
- Future enhancement: Add `cached_at` timestamp checks, manual refresh endpoint
- Database connection update triggers metadata re-fetch

### 5. Axum CORS Configuration

**Task**: Research proper CORS configuration in Axum to allow all origins.

**Decision**: Use **tower-http CorsLayer** with permissive settings

**Implementation**:
```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

let app = Router::new()
    .route("/api/v1/databases", get(list_databases).post(add_database))
    // ... more routes
    .layer(cors);
```

**Rationale**:
- `tower-http` is the standard middleware for Axum (built on Tower)
- `allow_origin(Any)` permits all origins (per constitution requirement)
- `allow_methods(Any)` supports all HTTP methods
- `allow_headers(Any)` prevents header-related CORS errors
- Applied as layer to entire router

**Security Note**:
- Permissive CORS is acceptable per constitution (trusted environment)
- Production deployment should be on trusted network
- No sensitive data exposure (no authentication means no user data)

### 6. OpenAI API Integration with openai_api_rust

**Task**: Research how to use openai_api_rust SDK for chat completion API.

**Decision**: Use **Chat Completion API with GPT-4**

**Implementation Pattern**:
```rust
use openai_api_rust::chat::{ChatApi, ChatBody};
use openai_api_rust::Auth;

let auth = Auth::from_env(); // Reads OPENAI_API_KEY
let openai = ChatApi::new(auth);

let body = ChatBody {
    model: "gpt-4".to_string(),
    messages: vec![
        Message {
            role: "system".to_string(),
            content: system_prompt_with_schema,
        },
        Message {
            role: "user".to_string(),
            content: natural_language_query,
        },
    ],
    temperature: Some(0.0), // Deterministic
    ..Default::default()
};

let response = openai.create_chat_completion(&body).await?;
let sql = response.choices[0].message.content.trim();
```

**Rationale**:
- `Auth::from_env()` reads OPENAI_API_KEY automatically
- GPT-4 chosen for better SQL generation quality (can use GPT-3.5-turbo for cost savings)
- Temperature=0 for consistent, conservative outputs
- System message contains schema and rules, user message has query

**Error Handling**:
- Network errors: return 503 Service Unavailable
- API key missing: return 500 with clear message at startup
- Rate limits: return 429 with retry-after suggestion
- Invalid responses: fallback to error message

## Summary

All technical clarifications resolved:

1. **SQL Parser**: sqlparser-rs for SELECT validation and LIMIT detection
2. **Metadata Retrieval**: information_schema queries for tables/columns/constraints
3. **LLM Prompts**: Structured prompt with schema context, temperature=0
4. **SQLite Schema**: Normalized design with connections, table_metadata, column_metadata tables
5. **CORS**: tower-http CorsLayer with allow_origin(Any)
6. **OpenAI Integration**: openai_api_rust with Chat Completion API, GPT-4

All decisions prioritize:
- Ergonomic Rust patterns (sqlparser-rs, tower-http, SQLx)
- Constitution compliance (CORS, SELECT-only, camelCase via serde)
- Performance (caching, indexes, zero-copy parsing)
- Reliability (validation, error handling, stable APIs)

Ready to proceed to Phase 1 (data-model.md, contracts/, quickstart.md).
