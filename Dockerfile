####################################################################################################
## Builder
####################################################################################################
FROM rust:1.81.0-slim-bookworm AS builder
RUN apt update && apt install -y libssl-dev pkg-config libz-dev libcurl4 libpq-dev
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

####################################################################################################
## Final image
####################################################################################################
FROM debian:bookworm-slim

RUN apt update && apt install -y postgresql iputils-ping curl libssl-dev pkg-config libz-dev libcurl4 libpq-dev
RUN update-ca-certificates

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /bot

# Copy our build
COPY --from=builder /bot/target/release/horus-bot ./

COPY --from=builder /bot/docker/start.sh ./

COPY --from=builder /usr/local/cargo/bin/diesel ./
COPY --from=builder /bot/migrations/ ./migrations/

COPY --from=builder /bot/resources/ ./resources/


# Use an unprivileged user.
USER bot:bot

CMD ["bash", "/bot/start.sh"]
