import path from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
	plugins: [react()],
	test: {
		environment: "jsdom",
		globals: true,
		setupFiles: ["./src/__tests__/setup.ts"],
		include: ["src/**/*.{test,spec}.{ts,tsx}"],
		coverage: {
			provider: "v8",
			reporter: ["text", "html", "lcov"],
			reportsDirectory: "./coverage",
			exclude: [
				"node_modules/",
				"dist/",
				"coverage/",
				"src-tauri/",
				"src/__tests__/",
				"src/components/ui/",
				"src/routeTree.gen.ts",
				"**/*.d.ts",
				"src/main.tsx",
				"src/vite-env.d.ts",
				"vite.config.ts",
				"vitest.config.ts",
				"src/types/**",
				"App.tsx",
				"test-settings.tsx",
				"__root.tsx",
			],
			thresholds: {
				statements: 80,
				branches: 80,
				functions: 80,
				lines: 80,
			},
		},
	},
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),
		},
	},
});
