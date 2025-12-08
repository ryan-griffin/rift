FROM docker.io/denoland/deno:latest as builder

RUN apt-get update && apt-get install -y nodejs && \
	rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY deno.json deno.lock package.json ./

RUN deno install --allow-scripts

COPY . .

ARG API_HOST
ARG API_PORT
ARG API_CONTAINER

ENV VITE_API_ADDRESS=${API_HOST}:${API_PORT}
ENV VITE_API_CONTAINER_ADDRESS=${API_CONTAINER}:${API_PORT}

RUN deno run -A npm:vinxi build

FROM docker.io/denoland/deno:latest

WORKDIR /app

COPY --from=builder /app/.output ./

ARG APP_PORT=${APP_PORT}

EXPOSE ${APP_PORT}

CMD ["deno", "task", "start"]
