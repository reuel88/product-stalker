import { describe, expect, it } from "vitest";
import Loader from "@/components/loader";
import { render } from "../../test-utils";

describe("Loader", () => {
	it("should render the loader container with correct classes", () => {
		const { container } = render(<Loader />);

		const loaderContainer = container.firstChild as HTMLElement;
		expect(loaderContainer).toHaveClass(
			"flex",
			"h-full",
			"items-center",
			"justify-center",
			"pt-8",
		);
	});

	it("should render with animate-spin class", () => {
		const { container } = render(<Loader />);

		const spinner = container.querySelector(".animate-spin");
		expect(spinner).toBeInTheDocument();
	});

	it("should render the Loader2 icon as SVG", () => {
		const { container } = render(<Loader />);

		// Lucide icons render as SVG elements
		const svg = container.querySelector("svg");
		expect(svg).toBeInTheDocument();
		expect(svg).toHaveClass("animate-spin");
	});

	it("should render SVG with lucide loader class", () => {
		const { container } = render(<Loader />);

		const svg = container.querySelector("svg");
		expect(svg).toHaveClass("lucide");
	});
});
