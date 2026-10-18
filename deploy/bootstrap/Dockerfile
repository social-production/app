# Bootstrap + iroh relay (kitsune2_bootstrap_srv, default features include iroh-relay).
# Railway terminates TLS, so the server listens on plain $PORT.
# Match the Nix toolchain (holonix main-0.7 ships Rust 1.95); newer rustc stops
# allowing the undefined host imports zome WASM relies on.
FROM rust:1.95.0-bookworm AS build
RUN cargo install kitsune2_bootstrap_srv --version 0.5.2 --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/local/cargo/bin/kitsune2-bootstrap-srv /usr/local/bin/
CMD ["sh", "-c", "exec kitsune2-bootstrap-srv --production --listen 0.0.0.0:${PORT:-8080}"]
