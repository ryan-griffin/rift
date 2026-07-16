FROM docker.io/oven/bun:latest AS builder

WORKDIR /app

COPY package.json bun.lock ./

RUN bun install

COPY . .

ARG API_HOST
ARG API_PORT
ARG API_CONTAINER

ENV VITE_API_ADDRESS=${API_HOST}:${API_PORT}
ENV VITE_API_CONTAINER_ADDRESS=${API_CONTAINER}:${API_PORT}

RUN bun run build

FROM docker.io/oven/bun:latest

WORKDIR /app

COPY --from=builder /app/.output ./

ARG APP_PORT=${APP_PORT}

EXPOSE ${APP_PORT}

CMD ["bun", "run", "server/index.mjs"]
