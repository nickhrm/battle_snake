# ---- build stage ----
FROM rust:1.90-slim-bookworm AS build
WORKDIR /app

# cache dependencies: build once with dummy sources
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > src/lib.rs \
    && echo "fn main() {}" > src/bin/sim.rs \
    && echo "fn main() {}" > src/bin/bench.rs \
    && if [ "$(uname -m)" = "x86_64" ]; then export RUSTFLAGS="-C target-cpu=x86-64-v2"; fi \
    && cargo build --release --bin honey_badger \
    && rm -rf src

COPY src ./src
# popcnt etc. on x86 (every server CPU since ~2010); plain build elsewhere (arm64)
RUN touch src/main.rs src/lib.rs \
    && if [ "$(uname -m)" = "x86_64" ]; then export RUSTFLAGS="-C target-cpu=x86-64-v2"; fi \
    && cargo build --release --bin honey_badger \
    && strip target/release/honey_badger

# ---- runtime stage ----
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 10001 snake
COPY --from=build /app/target/release/honey_badger /usr/local/bin/honey_badger
USER snake
ENV PORT=8000
EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/honey_badger", "--healthcheck"]
CMD ["/usr/local/bin/honey_badger"]
