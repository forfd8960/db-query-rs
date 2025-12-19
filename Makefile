.PHONY: help build run dev test clean check fmt clippy db-reset db-clean install-deps

# Default target
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development
dev: ## Run server in development mode with auto-reload (requires cargo-watch)
	cargo watch -x run

run: ## Run the server
	cargo run

build: ## Build the project in debug mode
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

# Testing
test: ## Run all tests
	cargo test

test-verbose: ## Run tests with verbose output
	cargo test -- --nocapture

test-unit: ## Run unit tests only
	cargo test --lib

# Code Quality
check: ## Check code compilation without building
	cargo check

fmt: ## Format code using rustfmt
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt -- --check

clippy: ## Run clippy linter
	cargo clippy -- -D warnings

clippy-fix: ## Auto-fix clippy warnings
	cargo clippy --fix

# Database
db-reset: db-clean run-migration ## Reset database (clean + run migrations)

db-clean: ## Remove SQLite database files
	@echo "Cleaning SQLite database..."
	@rm -rf db-query/
	@echo "Database cleaned!"

run-migration: ## Run database migrations (happens automatically on server start)
	@echo "Migrations will run automatically when server starts"
	@echo "Starting server to apply migrations..."
	@cargo run &
	@sleep 2
	@pkill -f "db-query-rs" || true
	@echo "Migrations applied!"

# Dependencies
install-deps: ## Install development dependencies
	@echo "Installing cargo-watch for auto-reload..."
	cargo install cargo-watch
	@echo "Installing cargo-edit for dependency management..."
	cargo install cargo-edit
	@echo "Done!"

update-deps: ## Update dependencies
	cargo update

# Cleaning
clean: ## Clean build artifacts
	cargo clean
	@rm -rf db-query/

clean-all: clean ## Clean everything including database
	@echo "All cleaned!"

# Docker (optional)
docker-build: ## Build Docker image
	docker build -t db-query-rs .

docker-run: ## Run in Docker container
	docker run -p 8000:8000 --env-file .env db-query-rs

# Environment
setup: ## Initial project setup
	@if [ ! -f .env ]; then \
		cp .env.example .env; \
		echo "Created .env file from .env.example"; \
		echo "Please edit .env and add your OPENAI_API_KEY"; \
	fi
	@echo "Setup complete!"

# Release
release: fmt clippy test build-release ## Prepare for release (fmt + clippy + test + build)
	@echo "Release build ready!"

# Quick start
start: setup ## Quick start (setup + run)
	@echo "Starting server..."
	@cargo run

# API Testing
test-api: ## Test API endpoints (requires server running and curl)
	@echo "Testing health endpoint..."
	@curl -s http://localhost:8000/health || echo "Server not running on port 8000"

# Development workflow
dev-setup: install-deps setup ## Complete development setup
	@echo "Development environment ready!"
	@echo "Run 'make dev' to start development server with auto-reload"

# Git hooks
install-hooks: ## Install git pre-commit hooks
	@echo '#!/bin/sh' > .git/hooks/pre-commit
	@echo 'make fmt-check' >> .git/hooks/pre-commit
	@echo 'make clippy' >> .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Git hooks installed!"
