import { describe, expect, it } from "vitest";
import { HomeView } from "@/modules/home/ui/views/home-view";
import { render, screen } from "../../test-utils";

describe("HomeView", () => {
	describe("rendering", () => {
		it("should render the ASCII art title", () => {
			const { container } = render(<HomeView />);

			// ASCII art is rendered in a pre element
			const preElement = container.querySelector("pre");
			expect(preElement).toBeInTheDocument();
			expect(preElement).toHaveClass("font-mono", "text-sm");
		});

		it("should contain BETTER T STACK text in ASCII art", () => {
			const { container } = render(<HomeView />);

			const preElement = container.querySelector("pre");
			// The ASCII art contains the text representation
			expect(preElement?.textContent).toContain("██████╗");
		});

		it("should render API Status section", () => {
			render(<HomeView />);

			expect(screen.getByText("API Status")).toBeInTheDocument();
		});

		it("should render API Status in a section with heading", () => {
			render(<HomeView />);

			const heading = screen.getByRole("heading", { name: "API Status" });
			expect(heading).toBeInTheDocument();
			expect(heading.tagName).toBe("H2");
		});
	});

	describe("layout", () => {
		it("should have container with max-width", () => {
			const { container } = render(<HomeView />);

			const wrapper = container.firstChild as HTMLElement;
			expect(wrapper).toHaveClass("container", "max-w-3xl");
		});

		it("should have proper padding", () => {
			const { container } = render(<HomeView />);

			const wrapper = container.firstChild as HTMLElement;
			expect(wrapper).toHaveClass("px-4", "py-2");
		});

		it("should have bordered section", () => {
			const { container } = render(<HomeView />);

			const section = container.querySelector("section");
			expect(section).toBeInTheDocument();
			expect(section).toHaveClass("rounded-lg", "border", "p-4");
		});
	});

	describe("styling", () => {
		it("should have overflow handling for ASCII art", () => {
			const { container } = render(<HomeView />);

			const preElement = container.querySelector("pre");
			expect(preElement).toHaveClass("overflow-x-auto");
		});

		it("should have proper heading styling", () => {
			render(<HomeView />);

			const heading = screen.getByRole("heading", { name: "API Status" });
			expect(heading).toHaveClass("font-medium", "mb-2");
		});
	});
});
