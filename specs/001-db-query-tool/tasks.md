# Tasks: Database Query Tool

**Input**: Design documents from `/specs/001-db-query-tool/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: This project does not require TDD. Tests are created as part of implementation tasks.

**Organization**: Tasks consolidated into 3 phases for simplicity. Core features (US1-US3) bundled in Phase 2, advanced features (US4) in Phase 3.

## Format: `- [ ] [ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Single project structure: `src/`, `tests/` at repository root (as defined in plan.md)

---

## Phase 1: Setup & Foundation

**Purpose**: Project initialization, core infrastructure, and shared dependencies required by all features

**Duration Estimate**: 1-2 hours

- [ ] T001 Initialize Rust project with Cargo.toml dependencies (axum, sqlx, tokio, serde, tower-http, openai_api_rust, sqlparser)
- [ ] T002 Create project directory structure per plan.md (src/{models,services,api,storage,validation}, tests/{integration,unit}, migrations/)
- [ ] T003 [P] Create .env.example file with OPENAI_API_KEY and PORT variables
- [ ] T004 [P] Create README.md with quickstart instructions based on quickstart.md
- [ ] T005 Create SQLite schema in migrations/001_initial_schema.sql (database_connections, table_metadata, column_metadata tables)
- [ ] T006 [P] Implement configuration module in src/config.rs to load environment variables
- [ ] T007 [P] Create error types module in src/models/error.rs for standardized error responses
- [ ] T008 [P] Create DTO types in src/api/dto.rs with serde camelCase configuration for all API request/response models
- [ ] T009 Implement SQLite repository in src/storage/repository.rs with connection pool setup
- [ ] T010 [P] Setup CORS middleware configuration in src/main.rs using tower-http
- [ ] T011 Create Axum router structure in src/api/routes.rs with endpoint placeholders
- [ ] T012 [P] Implement main.rs with server initialization, SQLite migration on startup, and graceful shutdown

**Checkpoint**: Foundation complete - can run server, SQLite initialized, routing structure ready

---

## Phase 2: Core Features (US1, US2, US3)

**Purpose**: Implement essential database query tool features - connections, metadata, query execution

**Duration Estimate**: 6-8 hours

### User Story 1: Database Connection Management (P1)

**Goal**: Users can add, list, and update PostgreSQL database connections

**Independent Test**: Add database via POST, list via GET, update via PUT - verify persistence in SQLite

- [ ] T013 [P] [US1] Create DatabaseConnection model in src/models/database.rs with validation
- [ ] T014 [US1] Implement database_service.rs: add_database() with PostgreSQL connection URL validation and test connection
- [ ] T015 [US1] Implement database_service.rs: list_databases() to retrieve all connections from SQLite with credentials masked
- [ ] T016 [US1] Implement database_service.rs: update_database() to modify connection URL in SQLite
- [ ] T017 [US1] Implement POST /api/v1/databases handler in src/api/handlers/database_handlers.rs
- [ ] T018 [P] [US1] Implement GET /api/v1/databases handler in src/api/handlers/database_handlers.rs
- [ ] T019 [P] [US1] Implement PUT /api/v1/databases/{db_name} handler in src/api/handlers/database_handlers.rs
- [ ] T020 [US1] Add integration tests in tests/integration/database_tests.rs for all database CRUD operations

### User Story 2: Database Metadata Browsing (P2)

**Goal**: Users can retrieve cached database schema metadata (tables, views, columns)

**Independent Test**: Request metadata for configured database, verify tables/columns returned in camelCase JSON with caching

- [ ] T021 [P] [US2] Create TableMetadata and ColumnMetadata models in src/models/metadata.rs
- [ ] T022 [US2] Implement metadata_service.rs: fetch_metadata_from_postgres() using information_schema queries from research.md
- [ ] T023 [US2] Implement metadata_service.rs: cache_metadata_to_sqlite() to store tables and columns in SQLite
- [ ] T024 [US2] Implement metadata_service.rs: get_cached_metadata() to retrieve metadata from SQLite with fast joins
- [ ] T025 [US2] Implement metadata_service.rs: get_or_fetch_metadata() orchestration (check cache, fetch if missing, return camelCase)
- [ ] T026 [US2] Implement GET /api/v1/databases/{db_name}/metadata handler in src/api/handlers/metadata_handlers.rs
- [ ] T027 [US2] Add integration tests in tests/integration/metadata_tests.rs for metadata retrieval and caching behavior

### User Story 3: SQL Query Execution (P3)

**Goal**: Users can execute SELECT queries with automatic safety validation and LIMIT injection

**Independent Test**: Submit SELECT query, verify execution and camelCase results. Submit non-SELECT, verify rejection. Submit query without LIMIT, verify LIMIT 1000 added.

- [ ] T028 [P] [US3] Create QueryResult model in src/models/query.rs with camelCase serialization
- [ ] T029 [US3] Implement SQL validator in src/validation/sql_validator.rs using sqlparser-rs (parse, check SELECT-only, detect LIMIT)
- [ ] T030 [US3] Implement sql_validator.rs: inject_limit_if_missing() to append "LIMIT 1000" to queries without LIMIT
- [ ] T031 [US3] Implement query_service.rs: execute_query() with SQLx to run query on PostgreSQL and transform results to camelCase JSON
- [ ] T032 [US3] Implement POST /api/v1/databases/{db_name}/query handler in src/api/handlers/query_handlers.rs with validation pipeline
- [ ] T033 [US3] Add integration tests in tests/integration/query_tests.rs for valid queries, invalid queries, LIMIT injection, non-SELECT rejection
- [ ] T034 [P] [US3] Add unit tests in tests/unit/validation_tests.rs for SQL parser (SELECT detection, LIMIT detection, syntax errors)

**Checkpoint**: MVP complete - database connections, metadata browsing, safe query execution all working

---

## Phase 3: Advanced Features (US4)

**Purpose**: Natural language to SQL translation using OpenAI LLM

**Duration Estimate**: 3-4 hours

### User Story 4: Natural Language Query Translation (P4)

**Goal**: Users can submit natural language queries that get translated to SQL, validated, and executed

**Independent Test**: Submit "Get 10 users" in natural language, verify SQL generation, validation, execution, and result return with both generatedSql and result in response

- [ ] T035 [P] [US4] Create NaturalLanguageRequest model in src/models/query.rs
- [ ] T036 [US4] Implement llm_service.rs: initialize OpenAI client with Auth::from_env() for OPENAI_API_KEY
- [ ] T037 [US4] Implement llm_service.rs: build_prompt_with_schema() to construct LLM prompt using database metadata from research.md template
- [ ] T038 [US4] Implement llm_service.rs: generate_sql_from_nl() to call OpenAI Chat Completion API with temperature=0 and extract SQL from response
- [ ] T039 [US4] Implement llm_service.rs: parse_llm_response() to extract SQL from markdown code fences and clean up formatting
- [ ] T040 [US4] Implement POST /api/v1/databases/{db_name}/nl-query handler in src/api/handlers/query_handlers.rs (get metadata, call LLM, validate SQL, execute, return result)
- [ ] T041 [US4] Add error handling for LLM service unavailable (503), invalid API key (500), and unsafe SQL generation (400)
- [ ] T042 [US4] Add integration tests in tests/integration/nlquery_tests.rs for natural language query success, LLM error handling, and generated SQL validation

**Checkpoint**: All features complete - natural language query support functional

---

## Dependencies & Execution Order

### User Story Dependencies

```
US1 (Database Connections) 
  ↓ (blocks)
US2 (Metadata Browsing) ← depends on database connections
  ↓ (blocks)
US3 (Query Execution) ← depends on connections, benefits from metadata
  ↓ (blocks)  
US4 (NL Query) ← depends on metadata (for LLM context) and query execution (to run generated SQL)
```

**Parallel Execution Opportunities**:
- Phase 1: T003, T004, T006, T007, T008, T010, T012 can run in parallel
- Phase 2 US1: T018, T019 can run in parallel after T017
- Phase 2 US2: T021 can run in parallel with US1 tasks
- Phase 2 US3: T028, T034 can run in parallel with earlier US3 tasks
- Phase 3 US4: T035, T041 can run in parallel with other US4 tasks

### Task Dependencies (Sequential)

**Phase 1 Critical Path**:
T001 → T002 → T005 → T009 → T011 → T012

**Phase 2 US1 Critical Path**:
T013 → T014 → T015 → T016 → T017 → T020

**Phase 2 US2 Critical Path**:
T021 → T022 → T023 → T024 → T025 → T026 → T027

**Phase 2 US3 Critical Path**:
T028 → T029 → T030 → T031 → T032 → T033

**Phase 3 US4 Critical Path**:
T035 → T036 → T037 → T038 → T039 → T040 → T042

---

## Implementation Strategy

### MVP Scope (Minimum Viable Product)

**Phase 1 + Phase 2 (US1-US3)** = Fully functional database query tool

- ✅ Database connection management
- ✅ Schema metadata browsing with caching
- ✅ Safe SQL query execution with validation
- ✅ Ready for production use in trusted environments

**Phase 3 (US4)** = Enhancement (Natural Language)

- Optional advanced feature
- Can be skipped if OpenAI integration not needed
- Adds significant user experience improvement

### Incremental Delivery

1. **After Phase 1**: Server runs, can be deployed (no features yet)
2. **After Phase 2 US1**: Users can manage database connections (first deliverable value)
3. **After Phase 2 US2**: Users can explore database schemas (adds significant value)
4. **After Phase 2 US3**: Core tool complete, production-ready (MVP milestone)
5. **After Phase 3 US4**: Full-featured tool with AI assistance (complete)

### Parallel Work Recommendations

**Team of 1**: Execute sequentially by phase
**Team of 2**: 
- Developer A: Phase 1 → Phase 2 US1 → Phase 2 US3 → Phase 3
- Developer B: Phase 1 (parallel tasks) → Phase 2 US2 → Phase 2 US3 tests

**Team of 3**:
- Developer A: Phase 1 → Phase 2 US1 → Phase 3
- Developer B: Phase 1 (parallel tasks) → Phase 2 US2
- Developer C: Phase 1 (parallel tasks) → Phase 2 US3

---

## Validation Checklist

After each phase, verify:

### Phase 1 Complete ✓
- [ ] `cargo build` succeeds with all dependencies
- [ ] Server starts and listens on configured port
- [ ] SQLite database created at ./db-query/db_query.db
- [ ] CORS middleware active
- [ ] Health check endpoint responds (if implemented)

### Phase 2 Complete ✓
- [ ] Can add database connection via POST /api/v1/databases/
- [ ] Can list databases via GET /api/v1/databases/
- [ ] Can update database via PUT /api/v1/databases/{name}/
- [ ] Can retrieve metadata via GET /api/v1/databases/{name}/metadata/
- [ ] Metadata is cached in SQLite (second fetch is faster)
- [ ] Can execute SELECT query via POST /api/v1/databases/{name}/query/
- [ ] Non-SELECT queries are rejected with 400 error
- [ ] Queries without LIMIT get LIMIT 1000 appended
- [ ] All responses use camelCase field names

### Phase 3 Complete ✓
- [ ] Can submit natural language query via POST /api/v1/databases/{name}/nl-query/
- [ ] LLM generates valid SQL from natural language
- [ ] Generated SQL is validated (SELECT-only check)
- [ ] Natural language query returns both generatedSql and result
- [ ] LLM errors (API down, invalid key) return appropriate HTTP status codes

---

## Summary

**Total Tasks**: 42
- Phase 1 (Setup & Foundation): 12 tasks
- Phase 2 (Core Features US1-US3): 22 tasks
  - US1: 8 tasks
  - US2: 7 tasks  
  - US3: 7 tasks
- Phase 3 (Advanced US4): 8 tasks

**Estimated Duration**: 10-14 hours total
- Phase 1: 1-2 hours
- Phase 2: 6-8 hours
- Phase 3: 3-4 hours

**MVP Milestone**: After Phase 2 (34 tasks) - fully functional database query tool

**Parallel Opportunities**: 15 tasks marked with [P] can run in parallel with others

**Technology Stack**: Rust, Axum, SQLx, SQLite, PostgreSQL, openai_api_rust, sqlparser-rs, tower-http, serde
