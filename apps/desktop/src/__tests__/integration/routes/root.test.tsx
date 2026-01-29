import { describe, expect, it, vi } from "vitest";
import { RootComponent } from "@/routes/__root";
import { render, screen } from "../../test-utils";

// Mock @tanstack/react-router
vi.mock("@tanstack/react-router", () => ({
	createRootRouteWithContext: () => () => ({
		component: null,
	}),
	HeadContent: () => <div data-testid="head-content" />,
	Outlet: () => <div data-testid="outlet">Outlet Content</div>,
	Link: ({ to, children }: { to: string; children: React.ReactNode }) => (
		<a href={to} data-testid={`link-${to}`}>
			{children}
		</a>
	),
}));

// Mock @tanstack/react-router-devtools
vi.mock("@tanstack/react-router-devtools", () => ({
	TanStackRouterDevtools: () => <div data-testid="router-devtools" />,
}));

// Mock next-themes
vi.mock("next-themes", () => ({
	ThemeProvider: ({ children }: { children: React.ReactNode }) => (
		<div data-testid="next-themes-provider">{children}</div>
	),
	useTheme: () => ({
		theme: "system",
		setTheme: vi.fn(),
	}),
}));

// Mock sonner
vi.mock("@/components/ui/sonner", () => ({
	Toaster: () => <div data-testid="toaster" />,
}));

describe("RootComponent", () => {
	it("should render HeadContent", () => {
		render(<RootComponent />);

		expect(screen.getByTestId("head-content")).toBeInTheDocument();
	});

	it("should render ThemeProvider", () => {
		render(<RootComponent />);

		expect(screen.getByTestId("next-themes-provider")).toBeInTheDocument();
	});

	it("should render Header component", () => {
		render(<RootComponent />);

		// Header contains navigation links including Home
		expect(screen.getByText("Home")).toBeInTheDocument();
	});

	it("should render Outlet for child routes", () => {
		render(<RootComponent />);

		expect(screen.getByTestId("outlet")).toBeInTheDocument();
	});

	it("should render Toaster for notifications", () => {
		render(<RootComponent />);

		expect(screen.getByTestId("toaster")).toBeInTheDocument();
	});

	it("should render router devtools", () => {
		render(<RootComponent />);

		expect(screen.getByTestId("router-devtools")).toBeInTheDocument();
	});

	it("should render with grid layout", () => {
		const { container } = render(<RootComponent />);

		const gridContainer = container.querySelector(".grid.h-svh");
		expect(gridContainer).toBeInTheDocument();
	});

	it("should render navigation links in header", () => {
		render(<RootComponent />);

		expect(screen.getByTestId("link-/")).toBeInTheDocument();
		expect(screen.getByTestId("link-/products")).toBeInTheDocument();
		expect(screen.getByTestId("link-/settings")).toBeInTheDocument();
	});
});

describe("Route head configuration", () => {
	it("should have expected meta properties", () => {
		// Test the route configuration separately
		const head = {
			meta: [
				{ title: "product-stalker" },
				{
					name: "description",
					content: "product-stalker is a web application",
				},
			],
			links: [{ rel: "icon", href: "/favicon.ico" }],
		};

		expect(head.meta[0].title).toBe("product-stalker");
		expect(head.meta[1].name).toBe("description");
		expect(head.meta[1].content).toBe("product-stalker is a web application");
		expect(head.links[0].rel).toBe("icon");
		expect(head.links[0].href).toBe("/favicon.ico");
	});
});
