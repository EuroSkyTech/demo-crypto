# Build the WASM client for web deployment
[working-directory: 'client']
client-build:
    wasm-pack build --target web --out-dir ../static/pkg --dev

# Build the WASM client for production deployment
[working-directory: 'client']
client-build-release:
    wasm-pack build --target web --out-dir ../static/pkg --release

# Start the development server with TLS
server-start:
    cargo run -p server

# Build client and start server for development
dev: client-build server-start

# Clean all build artifacts and generated files
clean:
    rm -rf static/pkg
    cargo clean

# Run clippy linter on all packages
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Check code formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all
