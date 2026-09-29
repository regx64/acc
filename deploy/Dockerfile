# acc-api and acc-worker in one image; the compose file picks the command.

FROM rust:1.94-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY migrations migrations
RUN cargo build --release --locked -p acc-api -p acc-worker \
 && cp target/release/acc-api target/release/acc-worker /usr/local/bin/

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home /nonexistent acc \
 && mkdir -p /var/cache/acc && chown acc /var/cache/acc
COPY --from=build /usr/local/bin/acc-api /usr/local/bin/acc-worker /usr/local/bin/
USER acc
ENV RUST_LOG=info
EXPOSE 8080
CMD ["acc-api"]
