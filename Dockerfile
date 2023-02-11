# This builder step will be discarded at the end of build.
FROM rust:1.67.0 AS builder

# Will create the app folder if it does not exist.
WORKDIR /app

# Install system dependencies (for linking, in this case).
RUN apt update && apt install lld clang -y

# Copy all from our directories into Docker.
COPY . .

# `sqlx`, by default, needs a database connection in compile time to assert the queries are correct.
# Here we have saved that information running `sqlx prepare` (saves to a file sqlx can read).
ENV SQLX_OFFLINE true

# Now that we have all our source code, we can build the binary.
RUN cargo build --release

# Runtime will take the compiled binary from the builder
# and only store that (much smaller image size than before).
FROM debian:bullseye-slim AS runtime

# Install OpenSSL for our dependencies and ca-certificates for TLS certificates
# when establishing HTTPS connections.
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/newsletter newsletter
# We also need the configuration files.
COPY configuration configuration

# For config parsing.
ENV APP_ENVIRONMENT production

# Executed with `docker run`.
ENTRYPOINT ["./newsletter"]