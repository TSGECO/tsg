.PHONY: help images
.DEFAULT_GOAL := help

help: ## Display this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

images: ## Convert PDF images to PNG format
	@echo "Building images..."
	@inkscape docs/tsg-echo.pdf --export-type=png --export-dpi=300 --export-filename=docs/tsgeco.png

test: ## Run tests
	@echo "Running tests..."
	@cargo nextest run --nocapture

clean: ## Remove build artifacts
	@cargo clean

check: ## Check the code for errors without building
	@cargo check

fmt: ## Format code
	cargo fmt

lint: ## Run linter
	cargo clippy

cli-doc: ## Run cli doc generation
	cargo run -- --markdown-help > docs/cli.md

pre-commit: ## Run pre-commit checks
	pre-commit run --all-files

tsg-pdf: ## Convert tsg format  to PDF
	pandoc docs/tsg.md -o docs/tsg.pdf

sanitize: test pre-commit ## Sanitize the repo
	@echo "Sanitizing Repo..."
