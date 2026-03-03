FROM lukemathwalker/cargo-chef:0.1.75-rust-alpine3.23 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apk add --no-cache openssl openssl-dev openssl-libs-static
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin bridge

FROM alpine AS runner
WORKDIR /app
COPY --from=builder /app/target/release/bridge /usr/local/bin/bridge

ENTRYPOINT ["/usr/local/bin/bridge"]
