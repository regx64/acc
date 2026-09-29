# go-judge with the toolchains for C, C++17, Python 3, Java and Rust.
# Builds natively for amd64 (local) and arm64 (Oracle A1).

FROM golang:1.26-bookworm AS build
ARG GO_JUDGE_VERSION=v1.13.0
RUN CGO_ENABLED=0 go install github.com/criyle/go-judge/cmd/go-judge@${GO_JUDGE_VERSION}

FROM debian:bookworm-slim
ARG RUST_VERSION=1.94.1
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      gcc g++ libc6-dev python3 openjdk-17-jdk-headless ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

# Rust under /usr/local so the sandbox's read-only /usr bind covers it.
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain ${RUST_VERSION} --no-modify-path \
 && ln -s "$(ls -d /usr/local/rustup/toolchains/*)/bin" /usr/local/rust-bin \
 && /usr/local/rust-bin/rustc --version \
 && chmod -R a+rX /usr/local/rustup

COPY --from=build /go/bin/go-judge /usr/local/bin/go-judge
COPY mount.yaml /etc/go-judge/mount.yaml
RUN if [ -e /lib64 ]; then echo '  - {type: bind, source: /lib64, target: /lib64, readonly: true}' >> /etc/go-judge/mount.yaml; fi

# Sandboxed programs see this PATH (see GO_JUDGE_PATH in the worker).
ENV ES_MOUNT_CONF=/etc/go-judge/mount.yaml \
    ES_HTTP_ADDR=:5050 \
    ES_RELEASE=true \
    ES_PARALLELISM=2
EXPOSE 5050
ENTRYPOINT ["/usr/local/bin/go-judge"]
