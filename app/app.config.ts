import { config } from "dotenv";
import { resolve } from "node:path";
import process from "node:process";
import { defineConfig } from "@solidjs/start/config";
import tailwindcss from "@tailwindcss/vite";
import solidSvg from "vite-plugin-solid-svg";

config({ path: resolve("../.env") });

process.env.VITE_ADDRESS = `${process.env.API_HOST}:${process.env.API_PORT}`;

const isTauri = process.env.TAURI_ENV_PLATFORM !== undefined;

export default defineConfig({
	ssr: !isTauri,
	vite: {
		plugins: [tailwindcss(), solidSvg()],
		clearScreen: false,
		server: {
			watch: {
				ignored: ["**/src-tauri/**"],
			},
		},
	},
	server: {
		preset: "deno_server",
		output: {
			dir: isTauri ? ".output-tauri" : ".output",
		},
	},
});
