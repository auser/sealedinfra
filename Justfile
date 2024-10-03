set dotenv-load := true


default:
    @just --list --justfile {{justfile()}}

# Install required tools
install-required:
    @echo "Installing tools"
    @echo "Installing Rust nightly toolchain"
    rustup toolchain install nightly

    cargo install mdbook
    cargo install cargo-watch --force

    @echo "Installing nextest"
    cargo install cargo-nextest

    @echo "Installing sqlx"
    cargo install sqlx-cli

# Install recommended tools
install-recommended: install-required
    @echo "Installing gitoxide"
    cargo install gitoxide
    @echo "Installing onefetch"
    cargo install onefetch

# Build the base devcontainer
devcontainer-build:
    docker build -t auser/sealedinfra-devcontainer -f .devcontainer/docker/Dockerfile.base .

# Run the server in dev mode
dev:
    @cargo watch -w src/server -w Cargo.toml -x "run server start"

# Run migrations
migrate-up:
    @sqlx migrate run

# Rollback the last migration
migrate-down:
    @sqlx migrate revert

migrate-reset:
    @sqlx database reset -y
