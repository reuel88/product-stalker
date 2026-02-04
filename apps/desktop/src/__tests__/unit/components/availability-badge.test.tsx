import { beforeEach, describe, expect, it, vi } from "vitest";
import { AvailabilityBadge } from "@/modules/products/ui/components/availability-badge";
import { render, screen } from "../../test-utils";

// Mock useRelativeTime hook
vi.mock("@/hooks/useRelativeTime", () => ({
	useRelativeTime: vi.fn((dateStr: string | null) => {
		if (!dateStr) return "";
		return "5m ago";
	}),
}));

describe("AvailabilityBadge", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	describe("status rendering", () => {
		it("should render 'Not checked' when status is null", () => {
			render(<AvailabilityBadge status={null} checkedAt={null} />);

			expect(screen.getByText("Not checked")).toBeInTheDocument();
		});

		it("should render 'In Stock' for in_stock status", () => {
			render(<AvailabilityBadge status="in_stock" checkedAt={null} />);

			expect(screen.getByText("In Stock")).toBeInTheDocument();
		});

		it("should render 'Out of Stock' for out_of_stock status", () => {
			render(<AvailabilityBadge status="out_of_stock" checkedAt={null} />);

			expect(screen.getByText("Out of Stock")).toBeInTheDocument();
		});

		it("should render 'Back Order' for back_order status", () => {
			render(<AvailabilityBadge status="back_order" checkedAt={null} />);

			expect(screen.getByText("Back Order")).toBeInTheDocument();
		});

		it("should render 'Unknown' for unknown status", () => {
			render(<AvailabilityBadge status="unknown" checkedAt={null} />);

			expect(screen.getByText("Unknown")).toBeInTheDocument();
		});
	});

	describe("styling", () => {
		it("should apply green styling for in_stock", () => {
			render(<AvailabilityBadge status="in_stock" checkedAt={null} />);

			const badge = screen.getByText("In Stock");
			expect(badge).toHaveClass("bg-green-100");
		});

		it("should apply red styling for out_of_stock", () => {
			render(<AvailabilityBadge status="out_of_stock" checkedAt={null} />);

			const badge = screen.getByText("Out of Stock");
			expect(badge).toHaveClass("bg-red-100");
		});

		it("should apply yellow styling for back_order", () => {
			render(<AvailabilityBadge status="back_order" checkedAt={null} />);

			const badge = screen.getByText("Back Order");
			expect(badge).toHaveClass("bg-yellow-100");
		});

		it("should apply gray styling for unknown", () => {
			render(<AvailabilityBadge status="unknown" checkedAt={null} />);

			const badge = screen.getByText("Unknown");
			expect(badge).toHaveClass("bg-gray-100");
		});
	});

	describe("relative time display", () => {
		it("should show relative time when checkedAt is provided", () => {
			render(
				<AvailabilityBadge
					status="in_stock"
					checkedAt="2024-01-15T10:00:00Z"
				/>,
			);

			expect(screen.getByText("5m ago")).toBeInTheDocument();
		});

		it("should not show relative time when checkedAt is null", () => {
			render(<AvailabilityBadge status="in_stock" checkedAt={null} />);

			expect(screen.queryByText("5m ago")).not.toBeInTheDocument();
		});
	});

	describe("refresh button", () => {
		it("should show refresh button when onCheck is provided", () => {
			const onCheck = vi.fn();
			render(
				<AvailabilityBadge
					status="in_stock"
					checkedAt={null}
					onCheck={onCheck}
				/>,
			);

			const button = screen.getByRole("button", { name: "Check availability" });
			expect(button).toBeInTheDocument();
		});

		it("should not show refresh button when onCheck is not provided", () => {
			render(<AvailabilityBadge status="in_stock" checkedAt={null} />);

			expect(
				screen.queryByRole("button", { name: "Check availability" }),
			).not.toBeInTheDocument();
		});

		it("should call onCheck when button is clicked", async () => {
			const onCheck = vi.fn();
			const { user } = render(
				<AvailabilityBadge
					status="in_stock"
					checkedAt={null}
					onCheck={onCheck}
				/>,
			);

			const button = screen.getByRole("button", { name: "Check availability" });
			await user.click(button);

			expect(onCheck).toHaveBeenCalledTimes(1);
		});

		it("should disable button when isChecking is true", () => {
			const onCheck = vi.fn();
			render(
				<AvailabilityBadge
					status="in_stock"
					checkedAt={null}
					onCheck={onCheck}
					isChecking={true}
				/>,
			);

			const button = screen.getByRole("button", { name: "Check availability" });
			expect(button).toBeDisabled();
		});

		it("should show spinner when isChecking is true", () => {
			const onCheck = vi.fn();
			const { container } = render(
				<AvailabilityBadge
					status="in_stock"
					checkedAt={null}
					onCheck={onCheck}
					isChecking={true}
				/>,
			);

			// Loader2 icon renders with animate-spin class
			const spinner = container.querySelector(".animate-spin");
			expect(spinner).toBeInTheDocument();
		});

		it("should show refresh icon when not checking", () => {
			const onCheck = vi.fn();
			render(
				<AvailabilityBadge
					status="in_stock"
					checkedAt={null}
					onCheck={onCheck}
					isChecking={false}
				/>,
			);

			// RefreshCw icon should be present (no animate-spin)
			const button = screen.getByRole("button", { name: "Check availability" });
			const svg = button.querySelector("svg");
			expect(svg).not.toHaveClass("animate-spin");
		});
	});
});
