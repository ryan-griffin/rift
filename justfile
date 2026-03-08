default:
	@just --list

dev-api:
	cd api && cargo run

start-api:
	cd api && cargo run --release

migrate-reset:
	cd api/migration && cargo run -- fresh

dev-web:
	#!/usr/bin/env bash
	trap 'kill 0' EXIT
	just dev-api & bun run --cwd=app dev

build-web:
	bun run --cwd=app build

start-web:
	#!/usr/bin/env bash
	trap 'kill 0' EXIT
	just start-api & bun run --cwd=app start

dev-desktop:
	#!/usr/bin/env bash
	trap 'kill 0' EXIT
	just dev-api & bun run --cwd=app tauri dev

build-desktop:
	bun run --cwd=app tauri build

fmt:
	cargo fmt
