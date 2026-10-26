FROM rust:latest
WORKDIR /usr/src/backend
COPY . .
RUN rustup default nightly-2024-05-14
RUN cargo build --release
CMD ["cargo", "run", "--release"]