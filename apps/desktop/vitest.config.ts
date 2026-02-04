import path from "node:path";
import react from "@vitejs/plugin-react";
import { defineConfig, defineProject } from "vitest/config";

const sharedCoverageExclude = [
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
];

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
			exclude: sharedCoverageExclude,
			thresholds: {
				statements: 80,
				branches: 80,
				functions: 80,
				lines: 80,
			},
		},
		projects: [
			defineProject({
				test: {
					name: "unit",
					include: ["src/__tests__/unit/**/*.{test,spec}.{ts,tsx}"],
					environment: "jsdom",
					globals: true,
					setupFiles: ["./src/__tests__/setup.ts"],
				},
				resolve: {
					alias: {
						"@": path.resolve(__dirname, "./src"),
					},
				},
			}),
			defineProject({
				test: {
					name: "integration",
					include: ["src/__tests__/integration/**/*.{test,spec}.{ts,tsx}"],
					environment: "jsdom",
					globals: true,
					setupFiles: ["./src/__tests__/setup.ts"],
				},
				resolve: {
					alias: {
						"@": path.resolve(__dirname, "./src"),
					},
				},
			}),
		],
	},
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),
		},
	},
});
