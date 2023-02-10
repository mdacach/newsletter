FROM rust:1.67.0

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

# Executed with `docker run`.
ENTRYPOINT ["./target/release/newsletter"]