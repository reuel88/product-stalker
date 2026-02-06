import { describe, expect, it, vi } from "vitest";
import { TimeRangeSelector } from "@/modules/products/ui/components/time-range-selector";
import { render, screen } from "../../test-utils";

describe("TimeRangeSelector", () => {
	describe("rendering", () => {
		it("should render all time range options", () => {
			const onChange = vi.fn();
			render(<TimeRangeSelector value="7d" onChange={onChange} />);

			expect(
				screen.getByRole("button", { name: "7 Days" }),
			).toBeInTheDocument();
			expect(
				screen.getByRole("button", { name: "30 Days" }),
			).toBeInTheDocument();
			expect(
				screen.getByRole("button", { name: "All Time" }),
			).toBeInTheDocument();
		});

		it("should have correct aria-label on group", () => {
			const onChange = vi.fn();
			render(<TimeRangeSelector value="7d" onChange={onChange} />);

			expect(
				screen.getByRole("group", { name: "Time range filter" }),
			).toBeInTheDocument();
		});
	});

	describe("selection state", () => {
		it("should mark 7d button as pressed when value is 7d", () => {
			const onChange = vi.fn();
			render(<TimeRangeSelector value="7d" onChange={onChange} />);

			expect(screen.getByRole("button", { name: "7 Days" })).toHaveAttribute(
				"aria-pressed",
				"true",
			);
			expect(screen.getByRole("button", { name: "30 Days" })).toHaveAttribute(
				"aria-pressed",
				"false",
			);
			expect(screen.getByRole("button", { name: "All Time" })).toHaveAttribute(
				"aria-pressed",
				"false",
			);
		});

		it("should mark 30d button as pressed when value is 30d", () => {
			const onChange = vi.fn();
			render(<TimeRangeSelector value="30d" onChange={onChange} />);

			expect(screen.getByRole("button", { name: "7 Days" })).toHaveAttribute(
				"aria-pressed",
				"false",
			);
			expect(screen.getByRole("button", { name: "30 Days" })).toHaveAttribute(
				"aria-pressed",
				"true",
			);
			expect(screen.getByRole("button", { name: "All Time" })).toHaveAttribute(
				"aria-pressed",
				"false",
			);
		});

		it("should mark All Time button as pressed when value is all", () => {
			const onChange = vi.fn();
			render(<TimeRangeSelector value="all" onChange={onChange} />);

			expect(screen.getByRole("button", { name: "7 Days" })).toHaveAttribute(
				"aria-pressed",
				"false",
			);
			expect(screen.getByRole("button", { name: "30 Days" })).toHaveAttribute(
				"aria-pressed",
				"false",
			);
			expect(screen.getByRole("button", { name: "All Time" })).toHaveAttribute(
				"aria-pressed",
				"true",
			);
		});
	});

	describe("interactions", () => {
		it("should call onChange with '7d' when 7 Days button is clicked", async () => {
			const onChange = vi.fn();
			const { user } = render(
				<TimeRangeSelector value="30d" onChange={onChange} />,
			);

			await user.click(screen.getByRole("button", { name: "7 Days" }));

			expect(onChange).toHaveBeenCalledWith("7d");
		});

		it("should call onChange with '30d' when 30 Days button is clicked", async () => {
			const onChange = vi.fn();
			const { user } = render(
				<TimeRangeSelector value="7d" onChange={onChange} />,
			);

			await user.click(screen.getByRole("button", { name: "30 Days" }));

			expect(onChange).toHaveBeenCalledWith("30d");
		});

		it("should call onChange with 'all' when All Time button is clicked", async () => {
			const onChange = vi.fn();
			const { user } = render(
				<TimeRangeSelector value="7d" onChange={onChange} />,
			);

			await user.click(screen.getByRole("button", { name: "All Time" }));

			expect(onChange).toHaveBeenCalledWith("all");
		});
	});
});
