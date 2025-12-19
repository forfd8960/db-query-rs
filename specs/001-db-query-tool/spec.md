# Feature Specification: Database Query Tool

**Feature Branch**: `001-db-query-tool`  
**Created**: 2025-12-19  
**Status**: Draft  
**Input**: User description: "Database query tool with metadata browsing, SQL execution, and natural language query support"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Database Connection Management (Priority: P1)

Users need to connect to PostgreSQL databases by providing connection URLs, view all configured databases, and update connection details as database credentials change.

**Why this priority**: Without database connections, no other features can function. This is the foundational capability that enables all subsequent functionality.

**Independent Test**: Can be fully tested by adding a database URL via API, retrieving the list of databases, and updating a database connection. Success is validated when database connection information is persisted and retrievable.

**Acceptance Scenarios**:

1. **Given** a valid PostgreSQL connection URL, **When** user submits the URL via POST /api/v1/databases/, **Then** the system stores the connection and returns success with database name
2. **Given** multiple databases are configured, **When** user requests GET /api/v1/databases/, **Then** the system returns a list of all database connections with names and URLs (credentials masked)
3. **Given** an existing database connection, **When** user submits updated URL via PUT /api/v1/databases/{db_name}/, **Then** the system updates the connection details and confirms success
4. **Given** an invalid connection URL format, **When** user attempts to add the database, **Then** the system returns a validation error with specific details about the format issue

---

### User Story 2 - Database Metadata Browsing (Priority: P2)

Users need to explore database structure by viewing all tables and views with their column information, data types, and constraints to understand the schema before writing queries.

**Why this priority**: Once connected, users need visibility into database structure to formulate queries. This provides the foundation for both manual SQL writing and natural language queries.

**Independent Test**: Can be tested by connecting to a database with known schema, retrieving metadata via API, and verifying that all tables, views, and column details are accurately represented in the response.

**Acceptance Scenarios**:

1. **Given** a connected database with tables and views, **When** user requests GET /api/v1/databases/{db_name}/metadata/, **Then** the system returns complete metadata including all tables, views, columns, data types, and constraints in camelCase JSON format
2. **Given** database metadata was previously retrieved, **When** user requests metadata again, **Then** the system returns cached metadata from SQLite without re-querying the database
3. **Given** a database connection that fails, **When** user requests metadata, **Then** the system returns a clear error message indicating connection failure
4. **Given** a database with no tables, **When** user requests metadata, **Then** the system returns an empty tables array with success status

---

### User Story 3 - SQL Query Execution (Priority: P3)

Users need to execute SELECT queries against their databases and view results in a structured format, with automatic query safety validation and result limiting.

**Why this priority**: This is the core query capability that enables users to retrieve and analyze data. Depends on database connection (P1) and benefits from metadata visibility (P2).

**Independent Test**: Can be tested by submitting valid and invalid SQL queries via API and verifying that only SELECT statements execute, LIMIT clauses are added when missing, results are returned in JSON format, and non-SELECT queries are rejected.

**Acceptance Scenarios**:

1. **Given** a valid SELECT query, **When** user submits POST /api/v1/databases/{db_name}/query/ with SQL, **Then** the system executes the query, returns results in camelCase JSON format with column names and row data
2. **Given** a SELECT query without LIMIT clause, **When** user submits the query, **Then** the system automatically appends "LIMIT 1000" before execution
3. **Given** a non-SELECT query (INSERT, UPDATE, DELETE, DROP), **When** user attempts to execute it, **Then** the system rejects the query with an error message stating only SELECT statements are allowed
4. **Given** a syntactically invalid SQL query, **When** user submits the query, **Then** the system validates syntax and returns a descriptive error message without executing
5. **Given** a query that returns more than 1000 rows without explicit LIMIT, **When** user executes the query, **Then** the system returns exactly 1000 rows with success status

---

### User Story 4 - Natural Language Query Translation (Priority: P4)

Users need to query databases using natural language descriptions, which the system translates to SQL using LLM, executes, and returns results.

**Why this priority**: This is an advanced convenience feature that builds upon SQL execution (P3) and metadata (P2). Not essential for basic functionality but significantly improves user experience.

**Independent Test**: Can be tested by submitting natural language queries via API, verifying that SQL is generated correctly using database metadata context, and that results match expectations for the query intent.

**Acceptance Scenarios**:

1. **Given** a natural language query like "Get the first 10 rows from users table", **When** user submits POST /api/v1/databases/{db_name}/nl-query/, **Then** the system uses LLM with metadata context to generate SQL, executes it, and returns results
2. **Given** database metadata is available, **When** LLM generates SQL from natural language, **Then** the system includes table and column information in the LLM prompt to improve accuracy
3. **Given** LLM generates a non-SELECT query, **When** system validates the generated SQL, **Then** the system rejects it and returns an error indicating the natural language request cannot be safely executed
4. **Given** LLM service is unavailable, **When** user submits a natural language query, **Then** the system returns a clear error message indicating the translation service is not available

---

### Edge Cases

- What happens when a database connection URL becomes invalid after being stored (credentials change, server down)?
- How does the system handle databases with thousands of tables (metadata size)?
- What occurs when a SQL query takes longer than expected to execute (timeout behavior)?
- How are special characters in table/column names handled in SQL generation?
- What happens when the SQLite metadata storage file becomes corrupted?
- How does the system handle concurrent queries to the same database?
- What occurs when OpenAI API key is missing or invalid?
- How are database connection credentials secured in SQLite storage?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept PostgreSQL connection URLs in the format `postgresql://user:pass@host:port/dbname` and persist them to SQLite storage
- **FR-002**: System MUST retrieve database metadata (tables, views, columns with types and constraints) from PostgreSQL system catalogs
- **FR-003**: System MUST cache database metadata in SQLite for reuse to avoid repeated database introspection queries
- **FR-004**: System MUST validate all SQL queries to ensure they contain only SELECT statements before execution
- **FR-005**: System MUST automatically append "LIMIT 1000" to any SELECT query that lacks a LIMIT clause
- **FR-006**: System MUST parse SQL syntax and return descriptive errors for malformed queries without attempting execution
- **FR-007**: System MUST return all query results in JSON format with camelCase field names
- **FR-008**: System MUST use LLM (via OpenAI API) to translate natural language queries into SQL statements
- **FR-009**: System MUST include database metadata (tables, columns, types) in LLM context when generating SQL from natural language
- **FR-010**: System MUST validate LLM-generated SQL using the same safety rules as user-submitted SQL (SELECT only, syntax check)
- **FR-011**: System MUST expose all functionality via REST API endpoints with CORS enabled for all origins
- **FR-012**: System MUST read OpenAI API key from OPENAI_API_KEY environment variable
- **FR-013**: System MUST store SQLite database file at `./db-query/db_query.db` relative to application root
- **FR-014**: System MUST return appropriate HTTP status codes (200 for success, 400 for validation errors, 500 for server errors)
- **FR-015**: System MUST support updating existing database connection URLs by database name

### Key Entities *(include if feature involves data)*

- **Database Connection**: Represents a configured PostgreSQL database with connection URL and name identifier. Stored in SQLite with URL, name, and creation timestamp.
- **Table Metadata**: Represents a table or view in the connected database with name, columns, and schema information. Cached in SQLite after initial retrieval.
- **Column Metadata**: Represents a column within a table/view with name, data type, constraints (nullable, primary key, foreign key). Part of table metadata.
- **Query Result**: Represents the output of an executed SQL query with column names and row data. Returned as JSON with camelCase formatting.
- **Natural Language Request**: Represents a user's query intention in plain language that gets translated to SQL via LLM.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can add a new database connection and retrieve its metadata within 10 seconds for databases with up to 100 tables
- **SC-002**: Cached metadata retrieval completes in under 500ms for databases with up to 1000 tables
- **SC-003**: SQL query validation correctly rejects 100% of non-SELECT statements without execution
- **SC-004**: Queries without LIMIT clauses automatically receive "LIMIT 1000" appended in 100% of cases
- **SC-005**: Natural language to SQL translation produces executable SELECT queries for 80% of common query patterns (e.g., "get rows", "show columns", "filter by condition")
- **SC-006**: System handles at least 10 concurrent query executions without blocking
- **SC-007**: Query results return in under 3 seconds for queries returning up to 1000 rows
- **SC-008**: API endpoints return appropriate error messages that identify the specific problem (invalid SQL, missing table, connection failed) in 100% of error cases

## Assumptions *(optional)*

- PostgreSQL is the primary and only supported database type initially; other databases may be added later
- Database connections are assumed to be within the same network or accessible from where the application runs
- Users understand basic SQL concepts even when using natural language queries
- OpenAI API key and account have sufficient quota for natural language query translation
- SQLite storage at `./db-query/db_query.db` is persistent and accessible with read/write permissions
- Frontend exists separately and will consume the REST API to display data in tree structures and tables
- Users are responsible for network-level security; no authentication is implemented by design
- Database credentials stored in SQLite are readable by anyone with file access (trusted environment)
- LLM-generated SQL may occasionally be incorrect; users can fall back to manual SQL entry

## Out of Scope *(optional)*

- Authentication and authorization mechanisms
- Multi-user support with separate workspaces or permissions
- Database connections to non-PostgreSQL databases (MySQL, SQL Server, Oracle, etc.)
- Query result export to file formats (CSV, Excel, PDF)
- Query history or saved queries functionality
- Visual query builder or drag-and-drop interface
- Real-time query monitoring or performance profiling
- Database schema modification capabilities (CREATE, ALTER, DROP tables)
- Transaction management or multi-statement execution
- Database backup or restore functionality
- Custom LLM model selection or configuration
- Query result pagination beyond the default 1000 row limit
