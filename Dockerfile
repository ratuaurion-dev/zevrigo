# syntax=docker/dockerfile:1
# Paket Docker Homeostasis Coin, hanya regtest (TSK-M33, Spesifikasi Docker v0.1, DEC-10 bagian 9).
# Satu image: homeostasis-node, homeostasis-explorer, homeostasis-miner, homeostasis-cli, dan antarmuka web explorer.
# Image dasar dikunci digest indeks (registry resmi Docker Hub), sehingga build ulang memakai dasar yang sama.
#
#   docker build -t homeostasis:regtest .      (atau: make docker-build)
#   docker compose up -d                        (lihat compose.yaml)

ARG RUST_IMAGE=rust:1.99.0-slim-trixie@sha256:2752b332db73fdbb7dc576f06c82ed1f312005784ef913d7e04a28f5f55dc581
ARG NODE_IMAGE=node:24.22.0-trixie-slim@sha256:a6155384446670e9719bdc4fd0ebe4d4b29fb32a8d6aa523da3547a6cfbcf22c
ARG RUNTIME_IMAGE=debian:trixie-slim@sha256:f6b08b09a05725f9431502d3b5714f5c4576e82c1a84fa1debe0c24a7e0c3ec4

# 1. Program Rust (release, Cargo.lock dikunci). rust-toolchain.toml dan .cargo tidak disalin: versi Rust dari image.
FROM ${RUST_IMAGE} AS rust-build
# RandomX (DEC-01, DEC-10 bagian 10): randomx-rs membangun kode C++ resmi lewat cmake, make, dan g++
# (image rust slim tidak berisi make).
RUN apt-get update \
 && apt-get install -y --no-install-recommends cmake make g++ \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps
COPY tests ./tests
RUN cargo build --release --locked \
      -p homeostasis-node -p homeostasis-explorer -p homeostasis-miner -p homeostasis-cli \
 && mkdir /out \
 && cp target/release/homeostasis-node target/release/homeostasis-explorer \
       target/release/homeostasis-miner target/release/homeostasis-cli /out/

# 2. Antarmuka web explorer (Next.js static export): paket dari package-lock.json tanpa skrip pasang.
FROM ${NODE_IMAGE} AS web-build
WORKDIR /web
COPY apps/explorer/web/package.json apps/explorer/web/package-lock.json ./
RUN npm ci --ignore-scripts --no-audit --no-fund
COPY apps/explorer/web ./
RUN npm run build

# 3. Runtime: Debian slim, pengguna biasa (uid 10001), data di volume /data.
FROM ${RUNTIME_IMAGE}
RUN groupadd --system --gid 10001 hsis \
 && useradd --system --uid 10001 --gid hsis --home-dir /data --no-create-home --shell /usr/sbin/nologin hsis \
 && mkdir -p /data /usr/share/homeostasis \
 && chown hsis:hsis /data
COPY --from=rust-build /out/ /usr/local/bin/
COPY --from=web-build /web/out /usr/share/homeostasis/web
USER hsis
WORKDIR /data
VOLUME ["/data"]
ENTRYPOINT ["homeostasis-node"]
CMD ["--regtest", "--datadir", "/data"]
