import { defineConfig } from "@solidjs/start/config";
import path from "node:path";

export default defineConfig({
	vite: {
		resolve: {
			alias: {
				"@ui": path.resolve(
					path.dirname(new URL(import.meta.url).pathname),
					"../../packages/ui/index.ts",
				),
			},
		},
	},
});
