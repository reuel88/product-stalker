import { describe, expect, it } from "vitest";
import { FullPageError } from "@/modules/shared/ui/components/full-page-error";
import { render, screen } from "../../test-utils";

describe("FullPageError", () => {
	describe("rendering", () => {
		it("should render title and description", () => {
			render(
				<FullPageError
					title="Something went wrong"
					description="Please try again later"
				/>,
			);

			expect(screen.getByText("Something went wrong")).toBeInTheDocument();
			expect(screen.getByText("Please try again later")).toBeInTheDocument();
		});

		it("should render the alert triangle icon", () => {
			const { container } = render(
				<FullPageError title="Error" description="Description" />,
			);

			const svg = container.querySelector("svg");
			expect(svg).toBeInTheDocument();
			expect(svg).toHaveClass("lucide");
		});
	});

	describe("styling", () => {
		it("should render with full-screen centering wrapper", () => {
			const { container } = render(
				<FullPageError title="Error" description="Description" />,
			);

			const wrapper = container.firstChild as HTMLElement;
			expect(wrapper).toHaveClass(
				"flex",
				"h-screen",
				"w-full",
				"items-center",
				"justify-center",
			);
		});

		it("should have flex-col for vertical layout", () => {
			const { container } = render(
				<FullPageError title="Error" description="Description" />,
			);

			const wrapper = container.firstChild as HTMLElement;
			expect(wrapper).toHaveClass("flex-col");
		});
	});

	describe("typography", () => {
		it("should render title with correct styling", () => {
			render(<FullPageError title="Error Title" description="Description" />);

			const title = screen.getByText("Error Title");
			expect(title.tagName).toBe("H6");
			expect(title).toHaveClass("font-medium", "text-lg");
		});

		it("should render description with correct styling", () => {
			render(<FullPageError title="Title" description="Error Description" />);

			const description = screen.getByText("Error Description");
			expect(description.tagName).toBe("P");
			expect(description).toHaveClass("text-sm");
		});
	});
});
