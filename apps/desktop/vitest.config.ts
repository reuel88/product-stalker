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
				// Build artifacts & dependencies
				"node_modules/",
				"dist/",
				"coverage/",

				// Config files
				"vite.config.ts",
				"vitest.config.ts",

				// Entry points & environment
				"src/main.tsx",
				"src/vite-env.d.ts",

				// Generated files
				"src/routeTree.gen.ts",

				// Type definitions (no runtime code)
				"**/*.d.ts",
				"src/types/**",
				"**/types.ts",

				// Test infrastructure
				"src/__tests__/",

				// Non-testable source
				"src-tauri/",
				"src/routes/**",
				"src/components/ui/",
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
