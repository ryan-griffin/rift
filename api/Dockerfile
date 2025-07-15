FROM docker.io/library/rust:latest as builder

WORKDIR /api

COPY . .

RUN cargo build --release

FROM docker.io/library/debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates \
	&& rm -rf /var/lib/apt/lists/*

WORKDIR /api

COPY --from=builder /api/target/release/api ./

ARG API_PORT

EXPOSE ${API_PORT}

CMD ["./api"]
