FROM docker.io/oven/bun:latest AS builder

WORKDIR /app

COPY package.json bun.lock ./

RUN bun install

COPY . .

RUN bun run vinxi build

FROM docker.io/oven/bun:latest

WORKDIR /app

COPY --from=builder /app/.output ./

EXPOSE 8080

CMD ["bun", "run", "server/index.mjs"]
