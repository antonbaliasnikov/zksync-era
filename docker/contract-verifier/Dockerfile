# syntax=docker/dockerfile:experimental
FROM ghcr.io/matter-labs/zksync-build-base:latest AS builder

WORKDIR /usr/src/zksync
COPY . .

RUN cargo build --manifest-path ./core/Cargo.toml --release --bin zksync_contract_verifier

FROM ghcr.io/matter-labs/zksync-runtime-base:latest

COPY --from=builder /usr/src/zksync/core/target/release/zksync_contract_verifier /usr/bin/

# CMD tail -f /dev/null
ENTRYPOINT ["zksync_contract_verifier"]
