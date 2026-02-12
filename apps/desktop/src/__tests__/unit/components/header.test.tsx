import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import Header from "@/modules/shared/ui/components/header";
import { render, screen } from "../../test-utils";

// Mock @tanstack/react-router
vi.mock("@tanstack/react-router", () => ({
	Link: ({ to, children }: { to: string; children: ReactNode }) => (
		<a href={to} data-testid={`link-${to}`}>
			{children}
		</a>
	),
}));

// Mock next-themes
vi.mock("next-themes", () => ({
	useTheme: () => ({
		theme: "system",
		setTheme: vi.fn(),
	}),
}));

describe("Header", () => {
	it("should render navigation links", () => {
		render(<Header />);

		expect(screen.getByTestId("link-/")).toBeInTheDocument();
		expect(screen.getByTestId("link-/products")).toBeInTheDocument();
		expect(screen.getByTestId("link-/test-settings")).toBeInTheDocument();
		expect(screen.getByTestId("link-/settings")).toBeInTheDocument();
	});

	it("should render Home link with text", () => {
		render(<Header />);

		const homeLink = screen.getByTestId("link-/");
		expect(homeLink).toHaveTextContent("Home");
	});

	it("should render product link with Package icon", () => {
		render(<Header />);

		const productLink = screen.getByTestId("link-/products");
		const svg = productLink.querySelector("svg");
		expect(svg).toBeInTheDocument();
		expect(svg).toHaveClass("size-4");
	});

	it("should render test-settings link with FlaskConical icon", () => {
		render(<Header />);

		const testSettingsLink = screen.getByTestId("link-/test-settings");
		const svg = testSettingsLink.querySelector("svg");
		expect(svg).toBeInTheDocument();
		expect(svg).toHaveClass("size-4");
	});

	it("should render settings link with Settings icon", () => {
		render(<Header />);

		const settingsLink = screen.getByTestId("link-/settings");
		const svg = settingsLink.querySelector("svg");
		expect(svg).toBeInTheDocument();
		expect(svg).toHaveClass("size-4");
	});

	it("should render ModeToggle component", () => {
		render(<Header />);

		// ModeToggle renders a button with sr-only text
		expect(screen.getByText("Toggle theme")).toBeInTheDocument();
	});

	it("should render horizontal rule separator", () => {
		const { container } = render(<Header />);

		const hr = container.querySelector("hr");
		expect(hr).toBeInTheDocument();
	});

	it("should render with correct layout classes", () => {
		const { container } = render(<Header />);

		const flexContainer = container.querySelector(
			".flex.flex-row.items-center.justify-between",
		);
		expect(flexContainer).toBeInTheDocument();
	});

	it("should render navigation with gap between links", () => {
		const { container } = render(<Header />);

		const nav = container.querySelector("nav");
		expect(nav).toBeInTheDocument();
		expect(nav).toHaveClass("flex", "gap-4", "text-lg");
	});
});
