import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import basicSsl from "@vitejs/plugin-basic-ssl"
export default defineConfig({
  build: {
    target: "esnext"
  },
  plugins: [basicSsl(), wasm()],
  worker: {
    plugins: () => [
      wasm()
    ]
  },
  server: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
  },
});
