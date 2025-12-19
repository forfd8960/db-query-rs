# Implementation Plan: Database Query Tool

**Branch**: `001-db-query-tool` | **Date**: 2025-12-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-db-query-tool/spec.md`

## Summary

Build a PostgreSQL database query tool that enables users to connect to databases, browse metadata (tables/views/columns), execute SQL queries with automatic safety validation, and translate natural language to SQL using LLM. The system will cache metadata in SQLite, validate all queries to SELECT-only with automatic LIMIT 1000, and expose all functionality via REST API with CORS enabled. Technical approach uses Rust/Axum/SQLx for backend services and openai_api_rust for natural language processing.

## Technical Context

**Language/Version**: Rust 1.75+ (stable, using 2021 edition)  
**Primary Dependencies**: Axum (web framework), SQLx (database library), openai_api_rust (LLM integration), Serde (JSON serialization), Tower-HTTP (CORS middleware), sqlparser-rs (SQL validation)  
**Storage**: SQLite for metadata cache (./db-query/db_query.db), PostgreSQL as target database for queries  
**Testing**: cargo test (unit/integration tests)  
**Target Platform**: Linux/macOS server, cross-platform binary
**Project Type**: Single backend service (no frontend, REST API only)  
**Performance Goals**: <10s metadata retrieval (100 tables), <500ms cached metadata, <3s query execution (1000 rows), 10+ concurrent queries  
**Constraints**: SELECT-only queries, automatic LIMIT 1000, no authentication, CORS for all origins, OPENAI_API_KEY environment variable required  
**Scale/Scope**: Developer tool for local/trusted networks, supports multiple database connections, metadata caching for efficiency

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Ergonomic Rust Code Style
✅ **PASS** - Feature design aligns with ergonomic Rust patterns. All code will use Result/Option types, pattern matching, and idiomatic Rust conventions. No violations.

### II. camelCase Data Format (NON-NEGOTIABLE)
✅ **PASS** - All API responses will use camelCase via serde rename_all configuration. Internal structs use snake_case, serialization outputs camelCase. Feature spec explicitly requires camelCase JSON responses (FR-007). No violations.

### III. Open Access - No Authentication
✅ **PASS** - Feature explicitly requires no authentication (FR-011, spec assumptions). CORS enabled for all origins. No user management or access control. Fully compliant with constitution principle III.

### Technology Stack Compliance
✅ **PASS** - Feature uses all required technologies from constitution: Rust, Axum, SQLx, openai_api_rust SDK, SQLite storage at ./db-query/db_query.db, PostgreSQL target database. Configuration via OPENAI_API_KEY environment variable as required.

### Security & Access Compliance
✅ **PASS** - Feature enforces query safety rules from constitution: SELECT-only validation (FR-004), automatic LIMIT 1000 (FR-005), SQL syntax validation (FR-006). All query safety requirements from constitution are implemented in feature requirements.

**Overall Status**: ✅ **ALL GATES PASS** - No constitution violations. Feature fully compliant with all core principles, technology stack requirements, and security rules. Ready to proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs                 # Application entry point, server setup
├── config.rs              # Configuration (env vars, app settings)
├── models/                # Data models and entities
│   ├── mod.rs
│   ├── database.rs        # DatabaseConnection entity
│   ├── metadata.rs        # TableMetadata, ColumnMetadata entities
│   └── query.rs           # QueryResult, NaturalLanguageRequest
├── services/              # Business logic
│   ├── mod.rs
│   ├── database_service.rs    # DB connection management
│   ├── metadata_service.rs    # Metadata retrieval & caching
│   ├── query_service.rs       # SQL execution & validation
│   └── llm_service.rs         # Natural language to SQL translation
├── api/                   # HTTP routing and handlers
│   ├── mod.rs
│   ├── routes.rs          # Route registration
│   ├── handlers/          # Request handlers
│   │   ├── mod.rs
│   │   ├── database_handlers.rs
│   │   ├── metadata_handlers.rs
│   │   └── query_handlers.rs
│   └── dto.rs             # Request/response DTOs with camelCase
├── storage/               # SQLite operations
│   ├── mod.rs
│   ├── schema.sql         # SQLite schema definition
│   └── repository.rs      # SQLite CRUD operations
└── validation/            # SQL validation & parsing
    ├── mod.rs
    └── sql_validator.rs   # SELECT-only, LIMIT injection

tests/
├── integration/           # Integration tests
│   ├── mod.rs
│   ├── api_tests.rs       # API endpoint tests
│   ├── database_tests.rs  # Database connection tests
│   └── query_tests.rs     # Query execution tests
└── unit/                  # Unit tests
    ├── mod.rs
    ├── validation_tests.rs
    └── service_tests.rs

migrations/                # SQLite schema migrations
└── 001_initial_schema.sql

Cargo.toml                 # Project dependencies
Cargo.lock
.env.example              # Example environment variables
README.md                 # Project documentation
```

**Structure Decision**: Single project structure selected because this is a backend-only REST API service with no frontend component. All code in `src/` organized by concern: models (entities), services (business logic), api (HTTP), storage (SQLite), validation (SQL safety). Tests separated into integration (API/DB) and unit (logic/validation).

## Complexity Tracking

*No constitution violations. This section intentionally empty.*

---

## Post-Phase 1 Constitution Re-Check

*GATE: Re-evaluate after Phase 1 design (research.md, data-model.md, contracts/, quickstart.md completed)*

### I. Ergonomic Rust Code Style
✅ **PASS** - Design maintains ergonomic Rust patterns. Data model uses standard Rust types (Result, Option). SQLite schema designed for SQLx compatibility. API handlers follow Axum patterns. No complex abstractions introduced.

### II. camelCase Data Format (NON-NEGOTIABLE)
✅ **PASS** - All API contracts in openapi.yaml specify camelCase field names. Data model explicitly documents camelCase serialization for all entities. Transformation rules defined for PostgreSQL snake_case → JSON camelCase. Fully compliant.

### III. Open Access - No Authentication
✅ **PASS** - No authentication mechanisms in API contracts. All endpoints publicly accessible. CORS configuration confirmed permissive (allow all origins). Database credentials stored without access control (trusted environment assumption maintained).

### Technology Stack Compliance
✅ **PASS** - Research decisions confirm all required technologies: Rust, Axum, SQLx, openai_api_rust, SQLite, PostgreSQL. Additional dependencies (sqlparser-rs, tower-http) align with constitution (ergonomic Rust libraries). OPENAI_API_KEY environment variable usage confirmed.

### Security & Access Compliance
✅ **PASS** - Research confirms sqlparser-rs for SELECT-only validation. Data model includes SQL validation service. API contracts document query safety (400 errors for non-SELECT). LIMIT injection strategy defined in research. All safety requirements implemented.

### Design Quality Assessment
✅ **PASS** - Data model is normalized, avoiding duplication. API contracts follow REST conventions. No over-engineering detected. Caching strategy simple and effective. Natural language feature properly isolated (can be disabled if LLM unavailable).

**Overall Status**: ✅ **ALL GATES PASS** - Phase 1 design fully compliant with constitution. No new violations introduced. Architecture aligns with ergonomic Rust principles, camelCase requirement, and security constraints. Ready to proceed to Phase 2 (tasks.md via /speckit.tasks command).

---

## Phase 0 & 1 Completion Summary

**✅ Phase 0: Research Complete**
- [research.md](research.md) - All technical clarifications resolved
  - SQL parser: sqlparser-rs selected
  - Metadata retrieval: information_schema queries
  - LLM prompts: Structured with schema context
  - SQLite schema: Normalized design with 3 tables
  - CORS: tower-http CorsLayer configuration
  - OpenAI integration: Chat Completion API pattern

**✅ Phase 1: Design Complete**
- [data-model.md](data-model.md) - 5 entities defined with relationships and state transitions
- [contracts/openapi.yaml](contracts/openapi.yaml) - OpenAPI 3.0 spec with 6 endpoints
- [quickstart.md](quickstart.md) - Developer onboarding guide with examples

**✅ Agent Context Updated**
- GitHub Copilot context file created with Rust/Axum/SQLx/OpenAI stack

**Next Step**: Run `/speckit.tasks` command to generate tasks.md (Phase 2) for implementation planning.
