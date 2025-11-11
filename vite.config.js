import { readFileSync } from "node:fs"
import { defineConfig } from "vite"
export default defineConfig({
	build: {
		target: "esnext",
		rollupOptions: {
			input: {
				main: "./index.html",
				sw: "./src/sw.ts",
			},
			output: {
				entryFileNames: "[name].js",
				assetFileNames: "[name].[ext]",
			},
		},
	},
	server: {
		https: process.env.CI
			? undefined
			: {
					key: readFileSync("./localhost-key.pem"),
					cert: readFileSync("./localhost.pem"),
				},
		headers: {
			"Cross-Origin-Opener-Policy": "same-origin",
			"Cross-Origin-Embedder-Policy": "require-corp",
			"Service-Worker-Allowed": "/",
			"Cache-Control": "no-cache, no-store, must-revalidate",
		},
	},
})
