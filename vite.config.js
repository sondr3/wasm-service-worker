import { readFileSync } from "node:fs"
import { defineConfig } from "vite"
import wasm from "vite-plugin-wasm"
export default defineConfig({
	base: process.env.CI ? undefined : process.env.VITE_BASE_PATH,
	build: {
		target: "esnext",
		rollupOptions: {
			input: {
				main: "./index.html",
				sw: "./src/sw.ts",
			},
			output: {
				entryFileNames: (chunkInfo) => {
					return chunkInfo.name === "sw" ? "sw.js" : "assets/[name]-[hash].js"
				},
			},
		},
	},
	plugins: [wasm()],
	worker: {
		plugins: () => [wasm()],
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
