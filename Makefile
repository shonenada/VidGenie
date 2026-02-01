##@ General

# The help target prints out all targets with their descriptions organized
# beneath their categories. The categories are represented by '##@' and the
# target descriptions by '##'. The awk commands is responsible for reading the
# entire set of makefiles included in this invocation, looking for lines of the
# file as xyz: ## something, and then pretty-format the target and help. Then,
# if there's a line with ##@ something, that gets pretty-printed as a category.
# More info on the usage of ANSI control characters for terminal formatting:
# https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters
# More info on the awk command:
# https://linuxcommand.org/lc3_adv_awk.php

.PHONY: help
help: ## Display help messages.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "	\033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Development

.PHONY: fmt
fmt: ## Format the project.
	cargo fmt --all

.PHONY: clean
clean: ## Clean the project.
	cargo clean

.PHONY: list-examples
list-examples: ## List available JSON examples.
	@echo "Available examples in examples/:"
	@ls -1 examples/*.json 2>/dev/null | xargs -n1 basename | sed 's/^/  /'

.PHONY: render-example
render-example: ## Render video from a JSON example (usage: make render-example EXAMPLE=filename.json)
	@test -n "$(EXAMPLE)" || (echo "Usage: make render-example EXAMPLE=filename.json"; echo "Run 'make list-examples' to see available examples."; exit 1)
	@test -f examples/$(EXAMPLE) || (echo "Error: examples/$(EXAMPLE) not found"; exit 1)
	@mkdir -p outputs
	@echo "Rendering $(EXAMPLE)..."
	@cargo run -p vg-cli -- --file "examples/$(EXAMPLE)" --output "outputs/$(shell basename $(EXAMPLE) .json).mp4"
	@echo "Video saved to outputs/$(shell basename $(EXAMPLE) .json).mp4"

.PHONY: render-all-examples
render-all-examples: ## Render videos from all JSON examples.
	@mkdir -p outputs
	@for file in examples/*.json; do \
		name=$$(basename "$$file" .json); \
		echo "Rendering $$name.json..."; \
		cargo run -p vg-cli -- --file "$$file" --output "outputs/$$name.mp4"; \
	done
	@echo "All videos saved to outputs/"

##@ Build

.PHONY: build
build: ## Build VidGenie.
	cargo build -p vg-cli 

.PHONY: docker-build
docker-build: ## Build Docker image for VidGenie (dockerfiles/build/Dockerfile).
	docker build --no-cache -f dockerfiles/build/Dockerfile -t vidgenie-build .

.PHONY: docker-render-example
docker-render-example: ## Render video from an example JSON inside Docker (usage: make docker-render-example EXAMPLE=filename.json)
	@test -n "$(EXAMPLE)" || (echo "Usage: make docker-render-example EXAMPLE=filename.json"; echo "Run 'make list-examples' to see available examples."; exit 1)
	@test -f examples/$(EXAMPLE) || (echo "Error: examples/$(EXAMPLE) not found"; exit 1)
	@mkdir -p outputs
	docker run --rm -v "$(PWD)":/workspace -w /workspace vidgenie-build make render-example EXAMPLE=$(EXAMPLE)

##@ Test

.PHONY: unit-test
unit-test: ## Run unit test against the project.
	cargo test --workspace

.PHONY: lint
lint: ## Run lint against the project.
	cargo check --workspace --all-targets || exit 1
	cargo clippy --workspace --all-targets

##@ Release

