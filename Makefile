APP_NAME := $(shell grep "name = " Cargo.toml | cut -d'"' -f2)
SOURCES_DIR := $(shell find . -type f -name "*.rs")
TEMPLATES_DIR := templates
JAVASCRIPT_DIR := js
ASSETS_DIR := assets
TARGET_APP := target/release/$(APP_NAME)
RELEASE_DIRECTORY :=target/release

.DEFAULT: help

.PHONY: help
help:
	@grep -E '^[///a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

$(TARGET_APP): $(SOURCES_DIR) $(TEMPLATES_DIR) $(JAVASCRIPT_DIR) $(ASSETS_DIR) ## Release version of the app
	@cargo build --release

.PHONY: check
check: ## Check code
	@cargo check

.PHONY: build
build: ## Build application (in release mode)
	@$(MAKE) -s $(TARGET_APP)

.PHONY: build-watch
build-watch: ## Automatic execution upon updates (in release mode)
	@find templates src js assets -type f | entr -r -s "$(MAKE) -s build & cargo run --release"

.PHONY: run
run: ## Run a release version
	@$(TARGET_APP)
