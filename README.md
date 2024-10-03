# Sealed Infrastructure

Sealed Infrastructure is a tool for managing Kubernetes infrastructure. It is designed to be used with the Sealed Kubernetes project.

## Getting Started

To get started with Sealed Infrastructure, you need to have the following prerequisites:

## CLI

The CLI is built with [clap](https://clap.rs/) and [tokio](https://tokio.rs/).

To build the CLI, run the following command:

```bash
cargo build --release
```

Run from installed binary:

```bash
si --root $PWD --settings ./config/config.yaml docker --repo git@bitbucket.org:financialpayments/tupay.git -b origin/eol/upgrade  build
```

Or from source:

```bash
RUST_LOG=debug cargo run -- --root $PWD --settings ./config/config.yaml docker --repo git@bitbucket.org:financialpayments/tupay.git -b origin/eol/upgrade  build
```

