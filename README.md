# Jito Vault Whitelist Program

## Instructions

- Initialize Config
- Initialize Whitelist

## Setup

### Generate IDL

```bash
cargo r -p shank-cli
```

### Generate Clients

```bash
pnpm generate-clients
```

### Test

```bash
cargo-build-sbf && SBF_OUT_DIR=$(pwd)/target/sbf-solana-solana/release cargo nextest run
```
