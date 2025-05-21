import { config } from "dotenv";
import { resolve } from "node:path";
import process from "node:process";
import { defineConfig } from "@solidjs/start/config";
import tailwindcss from "@tailwindcss/vite";
import solidSvg from "vite-plugin-solid-svg";

config({ path: resolve("../.env") });

const apiPort = +process.env.API_PORT!;
const host = process.env.TAURI_DEV_HOST;
const isTauri = process.env.TAURI_ENV_PLATFORM !== undefined;

export default defineConfig({
	ssr: !isTauri,
	vite: {
		plugins: [tailwindcss(), solidSvg()],
		// 1. prevent vite from obscuring rust errors
		clearScreen: false,
		// 2. tauri expects a fixed port, fail if that port is not available
		server: {
			proxy: {
				"/api": {
					target: `http://localhost:${apiPort}`,
					changeOrigin: true,
					rewrite: (path: string) => path.replace(/^\/api/, ""),
				},
			},
			port: 3000,
			strictPort: true,
			host: host || false,
			hmr: host
				? {
					protocol: "ws",
					host,
					port: 3001,
				}
				: undefined,
			watch: {
				// 3. tell vite to ignore watching `src-tauri`
				ignored: ["**/src-tauri/**"],
			},
		},
	},
});
