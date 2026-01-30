import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { HomeView } from "@/modules/home/ui/views/home-view";

describe("HomeComponent", () => {
	it("should render the ASCII art title", () => {
		const { container } = render(<HomeView />);

		// The ASCII art is rendered in a pre element
		const pre = container.querySelector("pre");
		expect(pre).toBeInTheDocument();
		// The ASCII art contains box drawing characters
		expect(pre?.textContent).toContain("█");
	});

	it("should render the API Status section", () => {
		render(<HomeView />);

		expect(screen.getByText("API Status")).toBeInTheDocument();
	});

	it("should render with container layout classes", () => {
		const { container } = render(<HomeView />);

		const mainContainer = container.querySelector(
			".container.mx-auto.max-w-3xl",
		);
		expect(mainContainer).toBeInTheDocument();
	});

	it("should render the ASCII art in a pre element with monospace font", () => {
		const { container } = render(<HomeView />);

		const pre = container.querySelector("pre");
		expect(pre).toBeInTheDocument();
		expect(pre).toHaveClass("font-mono", "text-sm", "overflow-x-auto");
	});

	it("should render the section with border styling", () => {
		const { container } = render(<HomeView />);

		const section = container.querySelector("section");
		expect(section).toBeInTheDocument();
		expect(section).toHaveClass("rounded-lg", "border", "p-4");
	});

	it("should render API Status heading with proper styling", () => {
		render(<HomeView />);

		const heading = screen.getByRole("heading", { name: "API Status" });
		expect(heading).toBeInTheDocument();
		expect(heading).toHaveClass("mb-2", "font-medium");
	});

	it("should render with a grid layout for sections", () => {
		const { container } = render(<HomeView />);

		const grid = container.querySelector(".grid.gap-6");
		expect(grid).toBeInTheDocument();
	});
});
