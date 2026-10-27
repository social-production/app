# Seed node: one process — the gateway with an embedded Holochain conductor. Conductor
# data, keystore and the gateway database live on a Railway volume at /data.
# Build from the repo root: docker build -f deploy/seed-node/Dockerfile .
ARG HOLOCHAIN_VERSION=0.7.1

# Match the Nix toolchain (holonix main-0.7 ships Rust 1.95.0) so zome WASM — and so the
# DNA hashes — are identical to desktop builds (docs/DEPLOYMENT.md#versions).
FROM rust:1.95.0-bookworm AS build
WORKDIR /src
RUN rustup target add wasm32-unknown-unknown
COPY . .
ARG HOLOCHAIN_VERSION
RUN curl -fsSL -o /usr/local/bin/hc \
      "https://github.com/holochain/holochain/releases/download/holochain-${HOLOCHAIN_VERSION}/hc-x86_64-unknown-linux-gnu" \
 && chmod +x /usr/local/bin/hc \
 && bash scripts/build-zomes.sh \
 && for dna in directory project conversation; do hc dna pack "dnas/$dna" -o "dnas/$dna/sp_$dna.dna"; done \
 && hc app pack happ -o happ/social_production.happ \
 && cargo build --release -p gateway --features jemalloc

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/gateway /usr/local/bin/gateway
COPY --from=build /src/happ/social_production.happ /app/social_production.happ
ENV DATA_DIR=/data HAPP_PATH=/app/social_production.happ RUST_LOG=info
CMD ["gateway"]
