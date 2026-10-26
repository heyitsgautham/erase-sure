# securewipe CLI. Build from the repository root:
#   docker build -f core/Dockerfile -t securewipe-cli .
#
# The container runs as an unprivileged user with no block devices mapped in,
# so it can discover, plan, sign, verify and export certificates but cannot wipe.
# Real wipes need root on the host (or the bootable ISO), never this image.

FROM rust:1.89-bookworm AS build
WORKDIR /src/core
COPY core/Cargo.toml core/Cargo.lock ./
COPY core/src src
RUN cargo build --release --locked --bin securewipe

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        util-linux \
        python3 python3-reportlab python3-qrcode python3-pil python3-jsonschema \
    && rm -rf /var/lib/apt/lists/*

# cert --export-pdf finds the project root by certs/ and core/python/generate_pdf.py,
# then runs that script, which imports the backup and wipe generators beside it.
WORKDIR /opt/securewipe
COPY certs/schemas certs/schemas
COPY core/python core/python
COPY --from=build /src/core/target/release/securewipe /usr/local/bin/securewipe

RUN useradd --create-home --uid 10001 securewipe
USER securewipe

ENTRYPOINT ["securewipe"]
CMD ["--help"]
