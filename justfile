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
	just dev-api & deno task --cwd=app dev

build-web:
	deno task --cwd=app build

start-web:
	#!/usr/bin/env bash
	trap 'kill 0' EXIT
	just start-api & deno task --cwd=app start

dev-desktop:
	#!/usr/bin/env bash
	trap 'kill 0' EXIT
	just dev-api & deno task --cwd=app tauri dev

build-desktop:
	deno task --cwd=app tauri build

fmt:
	cargo fmt
	deno fmt
