FROM rust:1.88.0-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY config ./config
COPY assets/maps ./assets/maps
COPY crates ./crates
RUN cargo build --locked --release -p brawler-server

FROM debian:bookworm-slim
COPY --from=build /app/target/release/brawler-server /usr/local/bin/brawler-server
USER 65534:65534
CMD ["brawler-server"]
