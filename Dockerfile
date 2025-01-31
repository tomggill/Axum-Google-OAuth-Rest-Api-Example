FROM rust:latest

WORKDIR /usr/src/lift-rust

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

RUN cargo install sqlx-cli

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

ENV SQLX_OFFLINE=true

RUN cargo build --release

COPY entrypoint.sh /usr/src/lift-rust/entrypoint.sh
RUN chmod +x /usr/src/lift-rust/entrypoint.sh

EXPOSE 3000

CMD ["./target/release/lift-rust"]
