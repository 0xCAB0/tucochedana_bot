####################################################################################################
## Builder
####################################################################################################
FROM rust:1.81.0-slim-bookworm AS builder
RUN apt update && apt install -y libssl-dev pkg-config libz-dev libcurl4 libpq-dev jq
RUN update-ca-certificates

# Create appuser
ENV USER=bot
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /bot

COPY ./ .

RUN cargo install cargo-cache
RUN cargo cache -a
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build --release

# Extract package name from metadata and save to PACKAGE_NAME file
RUN cargo metadata --format-version 1 --no-deps > target/cargo.toml.json && \
    jq -r '.packages[0].name' target/cargo.toml.json > target/PACKAGE_NAME

####################################################################################################
## Final image
####################################################################################################
FROM debian:bookworm-slim

RUN apt update && apt install -y postgresql iputils-ping curl libssl-dev pkg-config libz-dev libcurl4 libpq-dev jq
RUN update-ca-certificates

# Import user and group files from builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /bot

# Read package name from the file and copy the built binary using that name

# RUN export BOT_NAME=$(cat bot/target/PACKAGE_NAME) && \
#     cp bot/target/release/${BOT_NAME} ./

COPY --from=builder /bot/target/release/${BOT_NAME} ./

# Copy additional files
COPY --from=builder /bot/docker/start.sh ./
COPY --from=builder /usr/local/cargo/bin/diesel ./
COPY --from=builder /bot/migrations/ ./migrations/

# Use an unprivileged user.
USER bot:bot

CMD ["bash", "/bot/start.sh"]
