import { defineConfig } from "@solidjs/start/config";
import process from "node:process";
import tailwindcss from "@tailwindcss/vite";
import solidSvg from "vite-plugin-solid-svg";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	vite: {
		plugins: [tailwindcss(), solidSvg()],
		// 1. prevent vite from obscuring rust errors
		clearScreen: false,
		// 2. tauri expects a fixed port, fail if that port is not available
		server: {
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
