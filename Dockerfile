FROM rust:1.67.0

# Will create the app folder if it does not exist.
WORKDIR /app

# Install system dependencies (for linking, in this case).
RUN apt update && apt install lld clang -y

# Copy all from our directories into Docker.
COPY . .

# Now that we have all our source code, we can build the binary.
RUN cargo build --release

# Executed with `docker run`.
ENTRYPOINT ["./target/release/newsletter"]