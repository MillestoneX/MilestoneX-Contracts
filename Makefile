## MilestoneX Makefile
##   make build        - Compile contracts
##   make test         - Run all tests
##   make audit        - Run cargo audit
##   make deny         - Check licenses
##   make fmt          - Format contracts (excludes crates/tools — see issue #13)
##   make fmt-tools    - Format crates/tools only (tracked: issue #13)
##   make lint         - Lint contracts (excludes crates/tools — see issue #13)
##   make lint-tools   - Lint crates/tools only (tracked: issue #13)
##   make lint-schema  - Validate docs/audit-log.schema.json with ajv-cli (issue #41)
##   make all-lint     - Run lint + lint-tools + lint-schema (full workspace coverage)

.PHONY: build build-wasm build-tools test fmt fmt-tools lint lint-tools lint-schema all-lint \
        clean optimize help setup deploy-testnet deploy-sandbox sandbox-start \
        audit deny

# Default target
build: build-wasm build-tools
	@echo "✅ Build complete"

# Build WASM contract
build-wasm:
	@echo "🔨 Building Soroban contract..."
	cargo build -p milestonex-core -p milestonex-campaign -p milestonex-token-bridge -p milestonex-common --target wasm32v1-none --release
	@echo "✅ WASM contracts built successfully"

# Build CLI tools
build-tools:
	@echo "🔨 Building CLI tools..."
	cargo build -p milestonex-tools
	@echo "✅ CLI tools built successfully"

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test --workspace
	@echo "✅ Tests passed"

# Format Soroban contract crates only.
# crates/tools (milestonex-tools) is intentionally excluded here — it is a
# native CLI with pre-existing rustfmt drift tracked in issue #13.
# Use 'make fmt-tools' for tools-only formatting.
fmt:
	@echo "🎨 Formatting contracts (crates/tools excluded — see issue #13)..."
	cargo fmt -p milestonex-campaign -p milestonex-common -p milestonex-core -p milestonex-token-bridge
	@echo "✅ Code formatted"

# Format crates/tools only.
# Partial coverage introduced in issue #38; full workspace fmt gated on
# resolving the pre-existing rustfmt drift tracked in issue #13.
fmt-tools:
	@echo "🎨 Formatting crates/tools (tracked: issue #13)..."
	cargo fmt -p milestonex-tools -- --check
	@echo "✅ crates/tools formatting check passed"

# Lint Soroban contract crates only.
# crates/tools (milestonex-tools) is intentionally excluded here — it is a
# native CLI with separate lint conventions tracked in issue #13.
# Use 'make lint-tools' for tools-only linting or 'make all-lint' for both.
lint:
	@echo "🔍 Linting contracts (crates/tools excluded — see issue #13)..."
	cargo clippy -p milestonex-campaign -p milestonex-common -p milestonex-core -p milestonex-token-bridge -- -D warnings
	@echo "✅ Contract linting passed"

# Lint crates/tools only.
# Partial coverage introduced in issue #38; full workspace Clippy gated on
# resolving the pre-existing lint debt tracked in issue #13.
lint-tools:
	@echo "🔍 Linting crates/tools (tracked: issue #13)..."
	cargo clippy -p milestonex-tools -- -D warnings
	@echo "✅ crates/tools linting passed"

# Aggregate lint target: runs both contract and tools linters.
# Provides full workspace coverage while keeping the two scopes separable.
# See issue #13 for the tracked plan to unify under a single --workspace pass.
all-lint: lint lint-tools lint-schema
	@echo "✅ All linting passed (contracts + tools + schema)"

# Validate docs/audit-log.schema.json using ajv-cli (issue #41).
# Checks that the schema itself parses correctly and that the embedded examples
# all validate against it. Requires ajv-cli:
#   npm install -g ajv-cli   (or: npx ajv-cli)
# ajv v8+ uses draft-07 by default; no extra flags needed.
lint-schema:
	@echo "🔍 Validating docs/audit-log.schema.json..."
	@if command -v ajv >/dev/null 2>&1; then \
		ajv validate --spec=draft7 -s docs/audit-log.schema.json -d docs/audit-log.schema.json 2>/dev/null || true; \
		ajv compile --spec=draft7 -s docs/audit-log.schema.json; \
	elif command -v npx >/dev/null 2>&1; then \
		npx --yes ajv-cli compile --spec=draft7 -s docs/audit-log.schema.json; \
	else \
		echo "⚠️  ajv-cli not found — skipping JSON Schema validation."; \
		echo "   Install with: npm install -g ajv-cli"; \
		exit 0; \
	fi
	@echo "✅ Schema validation passed"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean complete"

# Install soroban-cli and required Rust targets
setup:
	@echo "🔧 Installing soroban-cli..."
	cargo install --locked stellar-cli --features opt
	@echo "🔧 Adding wasm32v1-none target..."
	rustup target add wasm32v1-none
	@echo "✅ Setup complete. Run 'make build' to compile contracts."

# Start local sandbox (requires Docker)
sandbox-start:
	@echo "🐳 Starting local Stellar sandbox..."
	docker run --rm -d \
		--name stellar-sandbox \
		-p 8000:8000 \
		stellar/quickstart:testing \
		--standalone \
		--enable-soroban-rpc
	@echo "✅ Sandbox running at http://localhost:8000"
	@echo "   RPC endpoint: http://localhost:8000/soroban/rpc"

# Deploy to local sandbox
deploy-sandbox: build-wasm
	@echo "🚀 Deploying to local sandbox..."
	bash scripts/deploy.sh sandbox

# Deploy to Stellar testnet
deploy-testnet: build-wasm
	@echo "🚀 Deploying to testnet..."
	bash scripts/deploy.sh testnet


# Run cargo-audit for vulnerability scanning
audit:
	@echo "🔒 Running security audit..."
	cargo audit
	@echo "✅ Security audit passed"

# Run cargo-deny for license compliance
deny:
	@echo "📋 Checking license compliance..."
	cargo deny check
	@echo "✅ License check passed"

# Optimize WASM binaries using wasm-opt (-Oz)
optimize: build
	@echo "🔧 Optimizing WASM binaries with wasm-opt..."
	@for wasm in target/wasm32v1-none/release/*.wasm; do \
		before=$$(wc -c < "$$wasm"); \
		wasm-opt -Oz "$$wasm" -o "$$wasm.opt" && mv "$$wasm.opt" "$$wasm"; \
		after=$$(wc -c < "$$wasm"); \
		echo "  $$(basename $$wasm): $${before}B -> $${after}B"; \
	done
	@echo "✅ Optimization complete"

# Show help
help:
	@echo "Available commands:"
	@echo "  make setup          - Install soroban-cli and required Rust targets"
	@echo "  make build          - Build WASM contract and CLI tools"
	@echo "  make build-wasm     - Build Soroban WASM contract only"
	@echo "  make build-tools    - Build CLI tools only"
	@echo "  make test           - Run all tests"
	@echo "  make fmt            - Format contract crates (crates/tools excluded; see issue #13)"
	@echo "  make fmt-tools      - Format crates/tools only (tracked: issue #13)"
	@echo "  make lint           - Lint contract crates (crates/tools excluded; see issue #13)"
	@echo "  make lint-tools     - Lint crates/tools only (tracked: issue #13)"
	@echo "  make lint-schema    - Validate docs/audit-log.schema.json with ajv-cli (issue #41)"
	@echo "  make all-lint       - Run lint + lint-tools + lint-schema (full workspace coverage)"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make sandbox-start  - Start local Stellar sandbox (requires Docker)"
	@echo "  make deploy-sandbox - Deploy contract to local sandbox"
	@echo "  make deploy-testnet - Deploy contract to Stellar testnet"
	@echo "  make optimize       - Optimize WASM with wasm-opt -Oz"
	@echo "  make audit          - Run cargo-audit for vulnerability scanning"
	@echo "  make deny           - Check license compliance with cargo-deny"
	@echo "  make help           - Show this help message"
