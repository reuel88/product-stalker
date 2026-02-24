import { describe, expect, it, vi } from "vitest";
import type { RetailerDetails } from "@/modules/products/price-utils";
import type { ProductRetailerResponse } from "@/modules/products/types";
import { RetailerList } from "@/modules/products/ui/components/retailer-list";
import { render, screen } from "../../test-utils";

vi.mock("@tauri-apps/plugin-opener", () => ({
	openUrl: vi.fn(),
}));

function createRetailer(
	overrides: Partial<ProductRetailerResponse> = {},
): ProductRetailerResponse {
	return {
		id: "pr-1",
		product_id: "product-1",
		retailer_id: "amazon.com",
		url: "https://www.amazon.com/dp/B123",
		label: null,
		sort_order: 0,
		created_at: new Date().toISOString(),
		...overrides,
	};
}

function createDetails(
	overrides: Partial<RetailerDetails> = {},
): RetailerDetails {
	return {
		priceMinorUnits: 9999,
		currency: "USD",
		currencyExponent: 2,
		status: "in_stock",
		checkedAt: new Date().toISOString(),
		todayAverageMinorUnits: null,
		yesterdayAverageMinorUnits: null,
		...overrides,
	};
}

describe("RetailerList", () => {
	const defaultProps = {
		retailers: [] as ProductRetailerResponse[],
		onRemove: vi.fn(),
		isRemoving: false,
		onReorder: vi.fn(),
	};

	it("should show empty message when no retailers", () => {
		render(<RetailerList {...defaultProps} />);

		expect(screen.getByText(/No retailers added yet/)).toBeInTheDocument();
	});

	it("should render retailer domain", () => {
		const retailers = [createRetailer()];
		render(<RetailerList {...defaultProps} retailers={retailers} />);

		expect(screen.getByText("www.amazon.com")).toBeInTheDocument();
	});

	it("should display price next to retailer when retailerDetails provided", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
				sort_order: 1,
			}),
		];
		const retailerDetails = new Map<string, RetailerDetails>([
			["pr-1", createDetails({ priceMinorUnits: 9999 })],
			["pr-2", createDetails({ priceMinorUnits: 7999 })],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerDetails={retailerDetails}
			/>,
		);

		expect(screen.getByText("$99.99")).toBeInTheDocument();
		expect(screen.getByText("$79.99")).toBeInTheDocument();
	});

	it("should not display price when retailerDetails not provided", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		render(<RetailerList {...defaultProps} retailers={retailers} />);

		expect(screen.queryByText("$99.99")).not.toBeInTheDocument();
	});

	it("should highlight cheapest retailer price in green", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
				sort_order: 1,
			}),
		];
		const retailerDetails = new Map<string, RetailerDetails>([
			["pr-1", createDetails({ priceMinorUnits: 9999 })],
			["pr-2", createDetails({ priceMinorUnits: 7999 })],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerDetails={retailerDetails}
				cheapestRetailerId="pr-2"
			/>,
		);

		const cheapestPrice = screen.getByText("$79.99");
		expect(cheapestPrice.className).toContain("text-green-600");

		const otherPrice = screen.getByText("$99.99");
		expect(otherPrice.className).not.toContain("text-green-600");
	});

	it("should not highlight when cheapestRetailerId is null", () => {
		const retailers = [createRetailer({ id: "pr-1" })];
		const retailerDetails = new Map<string, RetailerDetails>([
			["pr-1", createDetails({ priceMinorUnits: 9999 })],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerDetails={retailerDetails}
				cheapestRetailerId={null}
			/>,
		);

		const price = screen.getByText("$99.99");
		expect(price.className).not.toContain("text-green-600");
	});

	it("should skip price display for retailer without details", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
				sort_order: 1,
			}),
		];
		const retailerDetails = new Map<string, RetailerDetails>([
			["pr-1", createDetails({ priceMinorUnits: 9999 })],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerDetails={retailerDetails}
			/>,
		);

		expect(screen.getByText("$99.99")).toBeInTheDocument();
		// pr-2 has no details, so no second price element
		const priceElements = screen.getAllByText(/\$/);
		expect(priceElements).toHaveLength(1);
	});

	it("should display original price when currencies differ", () => {
		const retailers = [createRetailer({ id: "pr-1" })];
		const retailerDetails = new Map<string, RetailerDetails>([
			[
				"pr-1",
				createDetails({
					priceMinorUnits: 14250,
					currency: "AUD",
					currencyExponent: 2,
					originalPriceMinorUnits: 1430000,
					originalCurrency: "JPY",
					originalCurrencyExponent: 0,
				}),
			],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerDetails={retailerDetails}
			/>,
		);

		expect(screen.getByText("A$142.50")).toBeInTheDocument();
		expect(screen.getByText("\u00a51,430,000")).toBeInTheDocument();
	});

	it("should not display original price when currencies match", () => {
		const retailers = [createRetailer({ id: "pr-1" })];
		const retailerDetails = new Map<string, RetailerDetails>([
			[
				"pr-1",
				createDetails({
					priceMinorUnits: 18995,
					currency: "AUD",
					currencyExponent: 2,
				}),
			],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerDetails={retailerDetails}
			/>,
		);

		expect(screen.getByText("A$189.95")).toBeInTheDocument();
		// Only one price element should be rendered
		const allPriceTexts = screen.getAllByText(/A\$/);
		expect(allPriceTexts).toHaveLength(1);
	});

	it("should render drag handles for each retailer", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
				sort_order: 1,
			}),
		];

		render(<RetailerList {...defaultProps} retailers={retailers} />);

		expect(screen.getByTestId("drag-handle-pr-1")).toBeInTheDocument();
		expect(screen.getByTestId("drag-handle-pr-2")).toBeInTheDocument();
	});

	describe("StatusDot", () => {
		it("should show status dot when retailer has status", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				["pr-1", createDetails({ status: "in_stock" })],
			]);

			const { container } = render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			const dot = container.querySelector(".bg-green-500");
			expect(dot).toBeInTheDocument();
			expect(dot).toHaveAttribute("title", "In Stock");
		});

		it("should show red dot for out_of_stock", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				["pr-1", createDetails({ status: "out_of_stock" })],
			]);

			const { container } = render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			const dot = container.querySelector(".bg-red-500");
			expect(dot).toBeInTheDocument();
			expect(dot).toHaveAttribute("title", "Out of Stock");
		});

		it("should show yellow dot for back_order", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				["pr-1", createDetails({ status: "back_order" })],
			]);

			const { container } = render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			const dot = container.querySelector(".bg-yellow-500");
			expect(dot).toBeInTheDocument();
			expect(dot).toHaveAttribute("title", "Back Order");
		});

		it("should not show status dot when status is null", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				["pr-1", createDetails({ status: null })],
			]);

			const { container } = render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			expect(container.querySelector(".rounded-full.size-2")).toBeNull();
		});
	});

	describe("PriceChangeBadge", () => {
		it("should show price decrease badge", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				[
					"pr-1",
					createDetails({
						priceMinorUnits: 8000,
						todayAverageMinorUnits: 8000,
						yesterdayAverageMinorUnits: 10000,
					}),
				],
			]);

			render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			expect(screen.getByText("-20%")).toBeInTheDocument();
		});

		it("should show price increase badge", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				[
					"pr-1",
					createDetails({
						priceMinorUnits: 12000,
						todayAverageMinorUnits: 12000,
						yesterdayAverageMinorUnits: 10000,
					}),
				],
			]);

			render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			expect(screen.getByText("+20%")).toBeInTheDocument();
		});

		it("should not show badge when no yesterday data", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				[
					"pr-1",
					createDetails({
						priceMinorUnits: 8000,
						todayAverageMinorUnits: 8000,
						yesterdayAverageMinorUnits: null,
					}),
				],
			]);

			render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			expect(screen.queryByText(/%/)).not.toBeInTheDocument();
		});

		it("should not show badge when prices unchanged", () => {
			const retailers = [createRetailer({ id: "pr-1" })];
			const retailerDetails = new Map<string, RetailerDetails>([
				[
					"pr-1",
					createDetails({
						priceMinorUnits: 10000,
						todayAverageMinorUnits: 10000,
						yesterdayAverageMinorUnits: 10000,
					}),
				],
			]);

			render(
				<RetailerList
					{...defaultProps}
					retailers={retailers}
					retailerDetails={retailerDetails}
				/>,
			);

			expect(screen.queryByText(/%/)).not.toBeInTheDocument();
		});
	});
});
