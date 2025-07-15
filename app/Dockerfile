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

ENV API_HOST=${API_HOST}
ENV API_PORT=${API_PORT}
ENV API_CONTAINER=${API_CONTAINER}

RUN deno task build

FROM docker.io/denoland/deno:latest

WORKDIR /app

COPY --from=builder /app/.output ./

ARG PORT=${PORT}

EXPOSE ${PORT}

CMD ["deno", "task", "start"]
