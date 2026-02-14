import { describe, expect, it, vi } from "vitest";
import { formatDate } from "@/lib/format-date";
import type {
	AvailabilityCheckResponse,
	ProductResponse,
} from "@/modules/products/types";
import { ProductInfoCard } from "@/modules/products/ui/components/product-info-card";
import { render, screen } from "../../test-utils";

function createMockProduct(
	overrides: Partial<ProductResponse> = {},
): ProductResponse {
	return {
		id: "product-1",
		name: "Test Product",
		description: null,
		notes: null,
		currency: null,
		created_at: "2024-01-01T00:00:00Z",
		updated_at: "2024-01-15T00:00:00Z",
		...overrides,
	};
}

function createMockCheck(
	overrides: Partial<AvailabilityCheckResponse> = {},
): AvailabilityCheckResponse {
	return {
		id: "check-1",
		product_id: "product-1",
		product_retailer_id: null,
		status: "in_stock",
		raw_availability: null,
		error_message: null,
		checked_at: "2024-01-15T10:00:00Z",
		price_minor_units: 9999,
		price_currency: "USD",
		raw_price: "99.99",
		currency_exponent: 2,
		today_average_price_minor_units: null,
		yesterday_average_price_minor_units: null,
		is_price_drop: false,
		...overrides,
	};
}

describe("ProductInfoCard", () => {
	describe("product information", () => {
		it("should display product name", () => {
			const product = createMockProduct({ name: "My Product" });
			render(<ProductInfoCard product={product} latestCheck={null} />);

			expect(screen.getByText("My Product")).toBeInTheDocument();
		});

		it("should display product description when present", () => {
			const product = createMockProduct({ description: "A great product" });
			render(<ProductInfoCard product={product} latestCheck={null} />);

			expect(screen.getByText("A great product")).toBeInTheDocument();
		});

		it("should not display description section when null", () => {
			const product = createMockProduct({ description: null });
			render(<ProductInfoCard product={product} latestCheck={null} />);

			expect(screen.queryByText("Description")).not.toBeInTheDocument();
		});

		it("should display product notes when present", () => {
			const product = createMockProduct({ notes: "Buy when on sale" });
			render(<ProductInfoCard product={product} latestCheck={null} />);

			expect(screen.getByText("Buy when on sale")).toBeInTheDocument();
		});

		it("should not display notes section when null", () => {
			const product = createMockProduct({ notes: null });
			render(<ProductInfoCard product={product} latestCheck={null} />);

			expect(screen.queryByText("Notes")).not.toBeInTheDocument();
		});

		it("should display created date", () => {
			const product = createMockProduct({ created_at: "2024-01-01T00:00:00Z" });
			render(<ProductInfoCard product={product} latestCheck={null} />);

			const expectedDate = formatDate("2024-01-01T00:00:00Z");
			expect(
				screen.getByText(new RegExp(`Added: ${expectedDate}`)),
			).toBeInTheDocument();
		});

		it("should display updated date", () => {
			const product = createMockProduct({ updated_at: "2024-02-15T00:00:00Z" });
			render(<ProductInfoCard product={product} latestCheck={null} />);

			const expectedDate = formatDate("2024-02-15T00:00:00Z");
			expect(
				screen.getByText(new RegExp(`Updated: ${expectedDate}`)),
			).toBeInTheDocument();
		});
	});

	describe("availability status", () => {
		it("should display In Stock badge when status is in_stock", () => {
			const product = createMockProduct();
			const check = createMockCheck({ status: "in_stock" });
			render(<ProductInfoCard product={product} latestCheck={check} />);

			expect(screen.getByText("In Stock")).toBeInTheDocument();
		});

		it("should display Out of Stock badge when status is out_of_stock", () => {
			const product = createMockProduct();
			const check = createMockCheck({ status: "out_of_stock" });
			render(<ProductInfoCard product={product} latestCheck={check} />);

			expect(screen.getByText("Out of Stock")).toBeInTheDocument();
		});

		it("should display Back Order badge when status is back_order", () => {
			const product = createMockProduct();
			const check = createMockCheck({ status: "back_order" });
			render(<ProductInfoCard product={product} latestCheck={check} />);

			expect(screen.getByText("Back Order")).toBeInTheDocument();
		});

		it("should display Unknown badge when status is unknown", () => {
			const product = createMockProduct();
			const check = createMockCheck({ status: "unknown" });
			render(<ProductInfoCard product={product} latestCheck={check} />);

			expect(screen.getByText("Unknown")).toBeInTheDocument();
		});

		it("should not display status badge when latestCheck is null", () => {
			const product = createMockProduct();
			render(<ProductInfoCard product={product} latestCheck={null} />);

			expect(screen.queryByText("In Stock")).not.toBeInTheDocument();
			expect(screen.queryByText("Out of Stock")).not.toBeInTheDocument();
		});
	});

	describe("price display", () => {
		it("should display current price when available", () => {
			const product = createMockProduct();
			const check = createMockCheck({
				price_minor_units: 12999,
				price_currency: "USD",
			});
			render(<ProductInfoCard product={product} latestCheck={check} />);

			expect(screen.getByText("$129.99")).toBeInTheDocument();
		});

		it("should not display price section when price_cents is null", () => {
			const product = createMockProduct();
			const check = createMockCheck({
				price_minor_units: null,
				price_currency: null,
			});
			render(<ProductInfoCard product={product} latestCheck={check} />);

			expect(screen.queryByText("Current Price")).not.toBeInTheDocument();
		});
	});

	describe("refresh button", () => {
		it("should show refresh button when onCheck is provided", () => {
			const product = createMockProduct();
			const onCheck = vi.fn();
			render(
				<ProductInfoCard
					product={product}
					latestCheck={null}
					onCheck={onCheck}
				/>,
			);

			expect(
				screen.getByRole("button", { name: "Check availability" }),
			).toBeInTheDocument();
		});

		it("should not show refresh button when onCheck is not provided", () => {
			const product = createMockProduct();
			render(<ProductInfoCard product={product} latestCheck={null} />);

			expect(
				screen.queryByRole("button", { name: "Check availability" }),
			).not.toBeInTheDocument();
		});

		it("should call onCheck when refresh button is clicked", async () => {
			const product = createMockProduct();
			const onCheck = vi.fn();
			const { user } = render(
				<ProductInfoCard
					product={product}
					latestCheck={null}
					onCheck={onCheck}
				/>,
			);

			await user.click(
				screen.getByRole("button", { name: "Check availability" }),
			);

			expect(onCheck).toHaveBeenCalledTimes(1);
		});

		it("should disable refresh button when isChecking is true", () => {
			const product = createMockProduct();
			const onCheck = vi.fn();
			render(
				<ProductInfoCard
					product={product}
					latestCheck={null}
					onCheck={onCheck}
					isChecking={true}
				/>,
			);

			expect(
				screen.getByRole("button", { name: "Check availability" }),
			).toBeDisabled();
		});

		it("should show spinner when isChecking is true", () => {
			const product = createMockProduct();
			const onCheck = vi.fn();
			const { container } = render(
				<ProductInfoCard
					product={product}
					latestCheck={null}
					onCheck={onCheck}
					isChecking={true}
				/>,
			);

			const spinner = container.querySelector(".animate-spin");
			expect(spinner).toBeInTheDocument();
		});
	});
});
