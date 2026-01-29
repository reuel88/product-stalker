import { describe, expect, it, vi } from "vitest";
import { ThemeProvider, useTheme } from "@/components/theme-provider";
import { render, screen } from "../../test-utils";

// Mock next-themes
vi.mock("next-themes", () => ({
	ThemeProvider: ({
		children,
		...props
	}: { children: React.ReactNode } & Record<string, unknown>) => (
		<div data-testid="theme-provider" data-props={JSON.stringify(props)}>
			{children}
		</div>
	),
	useTheme: vi.fn(() => ({
		theme: "system",
		setTheme: vi.fn(),
		resolvedTheme: "light",
	})),
}));

describe("ThemeProvider", () => {
	it("should render children", () => {
		render(
			<ThemeProvider>
				<div data-testid="child">Child content</div>
			</ThemeProvider>,
		);

		expect(screen.getByTestId("child")).toBeInTheDocument();
		expect(screen.getByText("Child content")).toBeInTheDocument();
	});

	it("should pass props to NextThemesProvider", () => {
		render(
			<ThemeProvider
				attribute="class"
				defaultTheme="dark"
				storageKey="test-theme"
			>
				<div>Content</div>
			</ThemeProvider>,
		);

		const provider = screen.getByTestId("theme-provider");
		const props = JSON.parse(provider.getAttribute("data-props") || "{}");

		expect(props.attribute).toBe("class");
		expect(props.defaultTheme).toBe("dark");
		expect(props.storageKey).toBe("test-theme");
	});

	it("should render multiple children", () => {
		render(
			<ThemeProvider>
				<div data-testid="child-1">First</div>
				<div data-testid="child-2">Second</div>
			</ThemeProvider>,
		);

		expect(screen.getByTestId("child-1")).toBeInTheDocument();
		expect(screen.getByTestId("child-2")).toBeInTheDocument();
	});
});

describe("useTheme export", () => {
	it("should re-export useTheme from next-themes", () => {
		// useTheme should be defined and callable
		expect(useTheme).toBeDefined();
		expect(typeof useTheme).toBe("function");
	});

	it("should return theme properties when called", () => {
		const result = useTheme();

		expect(result).toHaveProperty("theme");
		expect(result).toHaveProperty("setTheme");
		expect(result).toHaveProperty("resolvedTheme");
	});
});
