import { describe, expect, it } from "vitest";
import { ErrorState } from "@/modules/shared/ui/components/error-state";
import { render, screen } from "../../test-utils";

describe("ErrorState", () => {
	describe("rendering", () => {
		it("should render with title only", () => {
			render(<ErrorState title="Something went wrong" />);

			expect(screen.getByText("Something went wrong")).toBeInTheDocument();
		});

		it("should render with description only", () => {
			render(<ErrorState description="Please try again later" />);

			expect(screen.getByText("Please try again later")).toBeInTheDocument();
		});

		it("should render with children", () => {
			render(
				<ErrorState>
					<button type="button">Retry</button>
				</ErrorState>,
			);

			expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
		});

		it("should render with all props", () => {
			render(
				<ErrorState title="Error Title" description="Error Description">
					<button type="button">Action</button>
				</ErrorState>,
			);

			expect(screen.getByText("Error Title")).toBeInTheDocument();
			expect(screen.getByText("Error Description")).toBeInTheDocument();
			expect(
				screen.getByRole("button", { name: "Action" }),
			).toBeInTheDocument();
		});

		it("should render without any props", () => {
			const { container } = render(<ErrorState />);

			// Should still render the container
			expect(container.firstChild).toBeInTheDocument();
		});
	});

	describe("icon", () => {
		it("should render the alert triangle icon", () => {
			const { container } = render(<ErrorState title="Error" />);

			// AlertTriangle icon renders as SVG with lucide class
			const svg = container.querySelector("svg");
			expect(svg).toBeInTheDocument();
			expect(svg).toHaveClass("lucide");
		});

		it("should have destructive text color on icon", () => {
			const { container } = render(<ErrorState title="Error" />);

			const svg = container.querySelector("svg");
			expect(svg).toHaveClass("text-destructive");
		});
	});

	describe("styling", () => {
		it("should have centered layout", () => {
			const { container } = render(<ErrorState title="Error" />);

			const wrapper = container.firstChild as HTMLElement;
			expect(wrapper).toHaveClass("flex", "items-center", "justify-center");
		});

		it("should have text centered", () => {
			const { container } = render(
				<ErrorState title="Error" description="Description" />,
			);

			const textContainer = container.querySelector(".text-center");
			expect(textContainer).toBeInTheDocument();
		});
	});

	describe("typography", () => {
		it("should render title with correct styling", () => {
			render(<ErrorState title="Error Title" />);

			const title = screen.getByText("Error Title");
			expect(title.tagName).toBe("H6");
			expect(title).toHaveClass("font-medium", "text-lg");
		});

		it("should render description with correct styling", () => {
			render(<ErrorState description="Error Description" />);

			const description = screen.getByText("Error Description");
			expect(description.tagName).toBe("P");
			expect(description).toHaveClass("text-sm");
		});
	});
});
