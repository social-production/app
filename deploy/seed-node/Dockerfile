# Seed node: Holochain conductor + gateway in one container, data on a Railway volume
# mounted at /data. Build from the repo root:
#   docker build -f deploy/seed-node/Dockerfile .
ARG HOLOCHAIN_VERSION=0.7.1

# Match the Nix toolchain (holonix main-0.7 ships Rust 1.95); newer rustc stops
# allowing the undefined host imports zome WASM relies on.
FROM rust:1.95.0-bookworm AS build
WORKDIR /src
RUN rustup target add wasm32-unknown-unknown
COPY . .
RUN cargo build --release -p gateway && bash scripts/build-zomes.sh
ARG HOLOCHAIN_VERSION
RUN curl -fsSL -o /usr/local/bin/hc \
      "https://github.com/holochain/holochain/releases/download/holochain-${HOLOCHAIN_VERSION}/hc-x86_64-unknown-linux-gnu" \
 && chmod +x /usr/local/bin/hc \
 && for dna in directory project conversation; do hc dna pack "dnas/$dna" -o "dnas/$dna/sp_$dna.dna"; done \
 && hc app pack happ -o happ/social_production.happ

FROM debian:bookworm-slim
ARG HOLOCHAIN_VERSION
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && curl -fsSL -o /usr/local/bin/holochain \
      "https://github.com/holochain/holochain/releases/download/holochain-${HOLOCHAIN_VERSION}/holochain-x86_64-unknown-linux-gnu" \
 && chmod +x /usr/local/bin/holochain
COPY --from=build /src/target/release/gateway /usr/local/bin/gateway
COPY --from=build /src/happ/social_production.happ /app/social_production.happ
COPY deploy/seed-node/conductor.yaml deploy/seed-node/entrypoint.sh /app/
RUN chmod +x /app/entrypoint.sh
ENV DATA_DIR=/data HAPP_PATH=/app/social_production.happ
ENTRYPOINT ["/app/entrypoint.sh"]
