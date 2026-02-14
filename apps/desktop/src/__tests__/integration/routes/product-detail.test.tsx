import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants";
import { ProductDetailView } from "@/modules/products/ui/views/product-detail-view";
import {
	createMockAvailabilityCheck,
	createMockProduct,
} from "../../mocks/data";
import {
	getMockedInvoke,
	mockInvokeError,
	mockInvokeMultiple,
} from "../../mocks/tauri";
import { render, screen, waitFor } from "../../test-utils";

describe("ProductDetailView", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	describe("loading state", () => {
		it("should show loading skeleton while fetching product", async () => {
			// Delay the response to see loading state
			getMockedInvoke().mockImplementation(() => new Promise(() => {}));

			const { container } = render(<ProductDetailView productId="product-1" />);

			// Should have skeleton elements (animate-pulse is used by Skeleton component)
			const skeletons = container.querySelectorAll("[class*='animate-pulse']");
			expect(skeletons.length).toBeGreaterThan(0);
		});
	});

	describe("error state", () => {
		it("should show error message when product fetch fails", async () => {
			mockInvokeError(COMMANDS.GET_PRODUCT, "Product not found");

			render(<ProductDetailView productId="nonexistent" />);

			await waitFor(() => {
				expect(screen.getByText("Product not found")).toBeInTheDocument();
			});
		});

		it("should show back to products link on error", async () => {
			mockInvokeError(COMMANDS.GET_PRODUCT, "Product not found");

			render(<ProductDetailView productId="nonexistent" />);

			await waitFor(() => {
				expect(
					screen.getByRole("link", { name: /back to products/i }),
				).toBeInTheDocument();
			});
		});
	});

	describe("product display", () => {
		it("should display product information when loaded", async () => {
			const product = createMockProduct({
				id: "product-1",
				name: "Test Product",
				description: "A great product",
				notes: "Buy when on sale",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Test Product")).toBeInTheDocument();
			});
			expect(screen.getByText("A great product")).toBeInTheDocument();
			expect(screen.getByText("Buy when on sale")).toBeInTheDocument();
		});

		it("should show back to products link", async () => {
			const product = createMockProduct({ id: "product-1" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(
					screen.getByRole("link", { name: /back to products/i }),
				).toBeInTheDocument();
			});
		});
	});

	describe("availability display", () => {
		it("should display current availability status", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				status: "in_stock",
				price_minor_units: 9999,
				price_currency: "USD",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("In Stock")).toBeInTheDocument();
			});
		});

		it("should display current price", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				status: "in_stock",
				price_minor_units: 12999,
				price_currency: "USD",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				// Price may appear multiple times (info card + chart single data point)
				const priceElements = screen.getAllByText("$129.99");
				expect(priceElements.length).toBeGreaterThan(0);
			});
		});
	});

	describe("price history chart", () => {
		it("should show price history section", async () => {
			const product = createMockProduct({ id: "product-1" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Price History")).toBeInTheDocument();
			});
			expect(
				screen.getByText("Track how the price has changed"),
			).toBeInTheDocument();
		});

		it("should show empty message when no price data", async () => {
			const product = createMockProduct({ id: "product-1" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("No price data available")).toBeInTheDocument();
			});
		});

		it("should show time range selector", async () => {
			const product = createMockProduct({ id: "product-1" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "7 Days" }),
				).toBeInTheDocument();
			});
			expect(
				screen.getByRole("button", { name: "30 Days" }),
			).toBeInTheDocument();
			expect(
				screen.getByRole("button", { name: "All Time" }),
			).toBeInTheDocument();
		});

		it("should filter price history when time range changes", async () => {
			const product = createMockProduct({ id: "product-1" });
			const now = new Date();
			const fiveDaysAgo = new Date(now.getTime() - 5 * 24 * 60 * 60 * 1000);
			const twentyDaysAgo = new Date(now.getTime() - 20 * 24 * 60 * 60 * 1000);

			const recentCheck = createMockAvailabilityCheck({
				id: "recent",
				checked_at: fiveDaysAgo.toISOString(),
				price_minor_units: 9999,
				price_currency: "USD",
			});
			const olderCheck = createMockAvailabilityCheck({
				id: "older",
				checked_at: twentyDaysAgo.toISOString(),
				price_minor_units: 8999,
				price_currency: "USD",
			});

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: recentCheck,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [recentCheck, olderCheck],
			});

			const { user } = render(<ProductDetailView productId="product-1" />);

			// Wait for initial render
			await waitFor(() => {
				expect(screen.getByText("Price History")).toBeInTheDocument();
			});

			// Default is 30d - should show both checks (chart rendered)
			// Switch to 7d
			await user.click(screen.getByRole("button", { name: "7 Days" }));

			// The chart behavior depends on filtered data, which we've verified in unit tests
			// Here we just verify the UI interaction works
			expect(screen.getByRole("button", { name: "7 Days" })).toHaveAttribute(
				"aria-pressed",
				"true",
			);
		});
	});

	describe("check availability", () => {
		it("should have a refresh button to check availability", async () => {
			const product = createMockProduct({ id: "product-1" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Check availability" }),
				).toBeInTheDocument();
			});
		});

		it("should trigger availability check when refresh button clicked", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				status: "in_stock",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
				[COMMANDS.CHECK_AVAILABILITY]: check,
			});

			const { user } = render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: "Check availability" }),
				).toBeInTheDocument();
			});

			await user.click(
				screen.getByRole("button", { name: "Check availability" }),
			);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.CHECK_AVAILABILITY,
					{ productId: "product-1" },
				);
			});
		});
	});

	describe("detailed price display", () => {
		it("should display detailed price indicator with price drop", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 79900,
				today_average_price_minor_units: 79900,
				yesterday_average_price_minor_units: 89900,
				price_currency: "USD",
				currency_exponent: 2,
			});

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Test Product product-1")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				const priceElements = screen.getAllByText("$799.00");
				expect(priceElements.length).toBeGreaterThan(0);
			});

			// Assert detailed comparison text (includes the "from" price)
			expect(screen.getByText(/Down.*from.*\$899\.00/)).toBeInTheDocument();

			// Assert green color class for price drop
			const comparisonElement = screen.getByText(/Down.*from/);
			expect(comparisonElement).toHaveClass(/text-green/);
		});

		it("should display detailed price indicator with price increase", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 89900,
				today_average_price_minor_units: 89900,
				yesterday_average_price_minor_units: 79900,
				price_currency: "USD",
				currency_exponent: 2,
			});

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Test Product product-1")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				const priceElements = screen.getAllByText("$899.00");
				expect(priceElements.length).toBeGreaterThan(0);
			});

			// Assert detailed comparison text (includes the "from" price)
			expect(screen.getByText(/Up.*from.*\$799\.00/)).toBeInTheDocument();

			// Assert red color class for price increase
			const comparisonElement = screen.getByText(/Up.*from/);
			expect(comparisonElement).toHaveClass(/text-red/);
		});

		it("should display price without comparison on first check", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 79900,
				today_average_price_minor_units: null,
				yesterday_average_price_minor_units: null,
				price_currency: "USD",
				currency_exponent: 2,
			});

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Test Product product-1")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				const priceElements = screen.getAllByText("$799.00");
				expect(priceElements.length).toBeGreaterThan(0);
			});

			// Assert no comparison text
			expect(screen.queryByText(/Down.*from/)).not.toBeInTheDocument();
			expect(screen.queryByText(/Up.*from/)).not.toBeInTheDocument();
		});

		it("should not show price section when price is null", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: null,
				price_currency: null,
				status: "out_of_stock",
			});

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Test Product product-1")).toBeInTheDocument();
			});

			// Wait for the availability to load, then verify no price section
			await waitFor(() => {
				// Verify "Current Price" label is not present
				expect(screen.queryByText("Current Price")).not.toBeInTheDocument();
			});

			// Verify no price information is displayed
			expect(screen.queryByText(/\$/)).not.toBeInTheDocument();
		});

		it("should display JPY currency without decimals in detailed view", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 1500,
				today_average_price_minor_units: null,
				yesterday_average_price_minor_units: null,
				price_currency: "JPY",
				currency_exponent: 0,
			});

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Test Product product-1")).toBeInTheDocument();
			});

			// Wait for JPY price to appear (availability query resolves async)
			await waitFor(() => {
				const priceElements = screen.getAllByText("¥1,500");
				expect(priceElements.length).toBeGreaterThan(0);
			});
		});

		it("should display large percentage change correctly", async () => {
			const product = createMockProduct({ id: "product-1" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 50000,
				today_average_price_minor_units: 50000,
				yesterday_average_price_minor_units: 100000,
				price_currency: "USD",
				currency_exponent: 2,
			});

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: product,
				[COMMANDS.GET_LATEST_AVAILABILITY]: check,
				[COMMANDS.GET_AVAILABILITY_HISTORY]: [check],
			});

			render(<ProductDetailView productId="product-1" />);

			await waitFor(() => {
				expect(screen.getByText("Test Product product-1")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				const currentPriceElements = screen.getAllByText("$500.00");
				expect(currentPriceElements.length).toBeGreaterThan(0);
			});

			// Assert comparison text with 50% change (includes the "from" price)
			expect(
				screen.getByText(/Down 50% from.*\$1,000\.00/),
			).toBeInTheDocument();
		});
	});
});
