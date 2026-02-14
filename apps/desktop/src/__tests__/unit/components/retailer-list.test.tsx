import { describe, expect, it, vi } from "vitest";
import type { RetailerPrice } from "@/modules/products/price-utils";
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
		created_at: new Date().toISOString(),
		...overrides,
	};
}

describe("RetailerList", () => {
	const defaultProps = {
		retailers: [] as ProductRetailerResponse[],
		onRemove: vi.fn(),
		isRemoving: false,
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

	it("should display price next to retailer when retailerPrices provided", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
			}),
		];
		const retailerPrices = new Map<string, RetailerPrice>([
			["pr-1", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
			["pr-2", { priceMinorUnits: 7999, currency: "USD", currencyExponent: 2 }],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerPrices={retailerPrices}
			/>,
		);

		expect(screen.getByText("$99.99")).toBeInTheDocument();
		expect(screen.getByText("$79.99")).toBeInTheDocument();
	});

	it("should not display price when retailerPrices not provided", () => {
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
			}),
		];
		const retailerPrices = new Map<string, RetailerPrice>([
			["pr-1", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
			["pr-2", { priceMinorUnits: 7999, currency: "USD", currencyExponent: 2 }],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerPrices={retailerPrices}
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
		const retailerPrices = new Map<string, RetailerPrice>([
			["pr-1", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerPrices={retailerPrices}
				cheapestRetailerId={null}
			/>,
		);

		const price = screen.getByText("$99.99");
		expect(price.className).not.toContain("text-green-600");
	});

	it("should skip price display for retailer without price data", () => {
		const retailers = [
			createRetailer({ id: "pr-1" }),
			createRetailer({
				id: "pr-2",
				url: "https://www.bestbuy.com/product/123",
			}),
		];
		const retailerPrices = new Map<string, RetailerPrice>([
			["pr-1", { priceMinorUnits: 9999, currency: "USD", currencyExponent: 2 }],
		]);

		render(
			<RetailerList
				{...defaultProps}
				retailers={retailers}
				retailerPrices={retailerPrices}
			/>,
		);

		expect(screen.getByText("$99.99")).toBeInTheDocument();
		// pr-2 has no price data, so no second price element
		const priceElements = screen.getAllByText(/\$/);
		expect(priceElements).toHaveLength(1);
	});
});
