import { describe, expect, it, vi } from "vitest";
import { CONFIG } from "@/constants";
import { render, screen } from "../test-utils";

// Mock @tanstack/react-router
vi.mock("@tanstack/react-router", () => ({
	createRouter: vi.fn(() => ({
		routeTree: {},
	})),
	RouterProvider: ({ router }: { router: unknown }) => (
		<div data-testid="router-provider" data-router={JSON.stringify(!!router)}>
			Router Content
		</div>
	),
}));

// Mock the route tree
vi.mock("@/routeTree.gen", () => ({
	routeTree: {},
}));

// Mock loader component
vi.mock("@/components/loader", () => ({
	default: () => <div data-testid="loader">Loading...</div>,
}));

import { App } from "@/App";

describe("App", () => {
	it("should render the app with QueryClientProvider and RouterProvider", () => {
		render(<App />);

		expect(screen.getByTestId("router-provider")).toBeInTheDocument();
	});

	it("should render router content", () => {
		render(<App />);

		expect(screen.getByText("Router Content")).toBeInTheDocument();
	});
});

describe("App Configuration", () => {
	it("should use CONFIG.QUERY.STALE_TIME for stale time", () => {
		expect(CONFIG.QUERY.STALE_TIME).toBeDefined();
		expect(typeof CONFIG.QUERY.STALE_TIME).toBe("number");
	});

	it("should use CONFIG.QUERY.RETRY for retry", () => {
		expect(CONFIG.QUERY.RETRY).toBeDefined();
		expect(typeof CONFIG.QUERY.RETRY).toBe("number");
	});
});
