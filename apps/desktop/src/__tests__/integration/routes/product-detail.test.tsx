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
				url: "https://example.com/product",
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
			expect(
				screen.getByText("https://example.com/product"),
			).toBeInTheDocument();
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
				price_cents: 9999,
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
				price_cents: 12999,
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
				price_cents: 9999,
				price_currency: "USD",
			});
			const olderCheck = createMockAvailabilityCheck({
				id: "older",
				checked_at: twentyDaysAgo.toISOString(),
				price_cents: 8999,
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
});
