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