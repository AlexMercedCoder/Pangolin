FROM rust:1.83 as builder

WORKDIR /usr/src/pangolin
COPY . .

RUN cargo build --release --bin pangolin_api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/pangolin/target/release/pangolin_api /usr/local/bin/pangolin
CMD ["pangolin"]
