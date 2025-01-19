FROM ghcr.io/matter-labs/zksync-build-base:latest AS chef
RUN cargo install cargo-chef --locked --version 0.1.70
WORKDIR /usr/src/zksync

FROM chef AS planner
COPY ./core .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/zksync/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY ./core .
RUN cargo build --release --bin zksync_contract_verifier

FROM ghcr.io/matter-labs/zksync-runtime-base:latest

COPY --from=builder /usr/src/zksync/target/release/zksync_contract_verifier /usr/bin/

# CMD tail -f /dev/null
ENTRYPOINT ["zksync_contract_verifier"]
