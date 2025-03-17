# Default recipe that shows help
default:
	@just --list

# Build the project
build:
	cargo build

# Build with optimizations
release:
	cargo build --release

# Remove build artifacts
clean:
	cargo clean

# Run tests
test:
	cargo nextest run --nocapture

# Check the code for errors without building
check:
	cargo check

# Run the application
run:
	cargo run

# Format code
fmt:
	cargo fmt

# Run linter
lint:
	cargo clippy

# Run cli doc generation
cli-doc:
	cargo run -- --markdown-help > docs/cli.md     

# Run pre-commit checks
pre-commit:
	pre-commit run --all-files

# Convert tsg format  to PDF 
tsg-pdf:
	pandoc docs/tsg.md -o docs/tsg.pdf