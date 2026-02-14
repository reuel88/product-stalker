import type { InvokeArgs } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { COMMANDS, MESSAGES } from "@/constants";
import { ProductsView } from "@/modules/products/ui/views/products-view";
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

// Mock sonner toast
vi.mock("sonner", () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn(),
		info: vi.fn(),
	},
}));

import { toast } from "sonner";

describe("ProductsComponent", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
		vi.clearAllMocks();
	});

	describe("loading state", () => {
		it("should show loading skeleton while fetching products", async () => {
			// Delay the response to see loading state
			getMockedInvoke().mockImplementation(() => new Promise(() => {}));

			render(<ProductsView />);

			// The ProductsTable shows skeleton when loading
			expect(screen.getByText("All Products")).toBeInTheDocument();
		});
	});

	describe("error state", () => {
		it("should show error message when products fail to load", async () => {
			mockInvokeError(COMMANDS.GET_PRODUCTS, "Failed to fetch");

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Failed to load products")).toBeInTheDocument();
			});
		});
	});

	describe("products list", () => {
		it("should render products when loaded", async () => {
			const products = [
				createMockProduct({ name: "Product One" }),
				createMockProduct({ name: "Product Two" }),
			];
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: products,
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Product One")).toBeInTheDocument();
				expect(screen.getByText("Product Two")).toBeInTheDocument();
			});
		});

		it("should show empty state when no products", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("No products found")).toBeInTheDocument();
			});
		});
	});

	describe("create product dialog", () => {
		it("should open create dialog when Add Product clicked", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Add Product")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Add Product"));

			expect(screen.getByRole("dialog")).toBeInTheDocument();
			expect(
				screen.getByText("Add a new product to track"),
			).toBeInTheDocument();
		});

		it("should create product successfully", async () => {
			const newProduct = createMockProduct({ name: "New Product" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
				[COMMANDS.CREATE_PRODUCT]: newProduct,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Add Product")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Add Product"));

			const nameInput = screen.getByLabelText("Name");

			await user.type(nameInput, "New Product");

			await user.click(screen.getByRole("button", { name: "Create" }));

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith(MESSAGES.PRODUCT.CREATED);
			});
		});

		it("should show validation error when name is missing", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Add Product")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Add Product"));
			await user.click(screen.getByRole("button", { name: "Create" }));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith(
					MESSAGES.VALIDATION.NAME_REQUIRED,
				);
			});
		});

		it("should show error toast when create fails", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([]);
				if (cmd === COMMANDS.CREATE_PRODUCT)
					return Promise.reject(new Error("Create failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Add Product")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Add Product"));

			const nameInput = screen.getByLabelText("Name");

			await user.type(nameInput, "Test");
			await user.click(screen.getByRole("button", { name: "Create" }));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith(
					MESSAGES.PRODUCT.CREATE_FAILED,
				);
			});
		});
	});

	describe("edit product dialog", () => {
		it("should open edit dialog with product data", async () => {
			const product = createMockProduct({
				name: "Existing Product",
				description: "A description",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Existing Product")).toBeInTheDocument();
			});

			// Open the dropdown menu
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			// Wait for menu to appear and click edit
			await waitFor(() => {
				expect(screen.getByText("Edit")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Edit"));

			expect(screen.getByRole("dialog")).toBeInTheDocument();
			expect(screen.getByText("Update product details")).toBeInTheDocument();
			expect(screen.getByDisplayValue("Existing Product")).toBeInTheDocument();
		});

		it("should update product successfully", async () => {
			const product = createMockProduct({ name: "Old Name" });
			const updatedProduct = { ...product, name: "New Name" };
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.UPDATE_PRODUCT]: updatedProduct,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Old Name")).toBeInTheDocument();
			});

			// Open menu and click edit
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			await waitFor(() => {
				expect(screen.getByText("Edit")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Edit"));

			// Clear and type new name
			const nameInput = screen.getByDisplayValue("Old Name");
			await user.clear(nameInput);
			await user.type(nameInput, "New Name");

			await user.click(screen.getByRole("button", { name: "Save" }));

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith(MESSAGES.PRODUCT.UPDATED);
			});
		});
	});

	describe("delete product dialog", () => {
		it("should open delete confirmation dialog", async () => {
			const product = createMockProduct({ name: "To Delete" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("To Delete")).toBeInTheDocument();
			});

			// Open menu and click delete
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Delete"));

			expect(screen.getByRole("dialog")).toBeInTheDocument();
			expect(
				screen.getByText(/Are you sure you want to delete/),
			).toBeInTheDocument();
			expect(screen.getByText(/"To Delete"/)).toBeInTheDocument();
		});

		it("should delete product successfully", async () => {
			const product = createMockProduct({ name: "To Delete" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.DELETE_PRODUCT]: undefined,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("To Delete")).toBeInTheDocument();
			});

			// Open menu and click delete
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});
			// Click the Delete menu item (not the dialog button)
			const deleteMenuItems = screen.getAllByText("Delete");
			await user.click(deleteMenuItems[0]);

			// Wait for dialog to appear then confirm deletion
			await waitFor(() => {
				expect(screen.getByRole("dialog")).toBeInTheDocument();
			});

			// Now click the Delete button in the dialog
			const deleteButtons = screen.getAllByRole("button", { name: "Delete" });
			await user.click(deleteButtons[deleteButtons.length - 1]); // Last one is the dialog button

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith(MESSAGES.PRODUCT.DELETED);
			});
		});

		it("should show error toast when delete fails", async () => {
			const product = createMockProduct({ name: "To Delete" });
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (cmd === COMMANDS.DELETE_PRODUCT)
					return Promise.reject(new Error("Delete failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("To Delete")).toBeInTheDocument();
			});

			// Open menu and click delete
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});
			const deleteMenuItems = screen.getAllByText("Delete");
			await user.click(deleteMenuItems[0]);

			// Wait for dialog to appear then confirm deletion
			await waitFor(() => {
				expect(screen.getByRole("dialog")).toBeInTheDocument();
			});

			const deleteButtons = screen.getAllByRole("button", { name: "Delete" });
			await user.click(deleteButtons[deleteButtons.length - 1]);

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith(
					MESSAGES.PRODUCT.DELETE_FAILED,
				);
			});
		});

		it("should cancel delete when Cancel clicked", async () => {
			const product = createMockProduct({ name: "To Delete" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("To Delete")).toBeInTheDocument();
			});

			// Open menu and click delete
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});
			const deleteMenuItems = screen.getAllByText("Delete");
			await user.click(deleteMenuItems[0]);

			// Wait for dialog then cancel
			await waitFor(() => {
				expect(screen.getByRole("dialog")).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: "Cancel" }));

			// Dialog should close
			await waitFor(() => {
				expect(
					screen.queryByText(/Are you sure you want to delete/),
				).not.toBeInTheDocument();
			});
		});
	});

	describe("price display in table", () => {
		it("should display compact price indicator with price drop", async () => {
			const product = createMockProduct({ id: "product-1", name: "Test" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 79900,
				today_average_price_minor_units: 79900,
				yesterday_average_price_minor_units: 89900,
				price_currency: "USD",
				currency_exponent: 2,
			});

			// Mock the invoke command handler
			getMockedInvoke().mockImplementation((cmd: string, args?: InvokeArgs) => {
				const typedArgs = args as { productId?: string } | undefined;

				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (
					cmd === COMMANDS.GET_LATEST_AVAILABILITY &&
					typedArgs?.productId === "product-1"
				) {
					return Promise.resolve(check);
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Test")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				expect(screen.getByText("$799.00")).toBeInTheDocument();
			});

			// Assert percentage change is visible with proper color (negative percentage for price drop)
			const percentElement = screen.getByText(/-\d+%/);
			expect(percentElement).toBeInTheDocument();
			expect(percentElement).toHaveClass(/text-green/);
		});

		it("should display compact price indicator with price increase", async () => {
			const product = createMockProduct({ id: "product-1", name: "Test" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 89900,
				today_average_price_minor_units: 89900,
				yesterday_average_price_minor_units: 79900,
				price_currency: "USD",
				currency_exponent: 2,
			});

			getMockedInvoke().mockImplementation((cmd: string, args?: InvokeArgs) => {
				const typedArgs = args as { productId?: string } | undefined;

				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (
					cmd === COMMANDS.GET_LATEST_AVAILABILITY &&
					typedArgs?.productId === "product-1"
				) {
					return Promise.resolve(check);
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Test")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				expect(screen.getByText("$899.00")).toBeInTheDocument();
			});

			// Assert percentage change is visible with proper color (positive percentage for price increase)
			const percentElement = screen.getByText(/\+\d+%/);
			expect(percentElement).toBeInTheDocument();
			expect(percentElement).toHaveClass(/text-red/);
		});

		it("should display price without comparison on first check", async () => {
			const product = createMockProduct({ id: "product-1", name: "Test" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 79900,
				today_average_price_minor_units: null,
				yesterday_average_price_minor_units: null,
				price_currency: "USD",
				currency_exponent: 2,
			});

			getMockedInvoke().mockImplementation((cmd: string, args?: InvokeArgs) => {
				const typedArgs = args as { productId?: string } | undefined;

				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (
					cmd === COMMANDS.GET_LATEST_AVAILABILITY &&
					typedArgs?.productId === "product-1"
				) {
					return Promise.resolve(check);
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Test")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				expect(screen.getByText("$799.00")).toBeInTheDocument();
			});

			// Assert no percentage indicators
			expect(screen.queryByText(/-\d+%/)).not.toBeInTheDocument();
			expect(screen.queryByText(/\+\d+%/)).not.toBeInTheDocument();
		});

		it("should display dash for null price", async () => {
			const product = createMockProduct({ id: "product-1", name: "Test" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: null,
				price_currency: null,
				status: "out_of_stock",
			});

			getMockedInvoke().mockImplementation((cmd: string, args?: InvokeArgs) => {
				const typedArgs = args as { productId?: string } | undefined;

				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (
					cmd === COMMANDS.GET_LATEST_AVAILABILITY &&
					typedArgs?.productId === "product-1"
				) {
					return Promise.resolve(check);
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Test")).toBeInTheDocument();
			});

			// Wait for dash to appear in price cell (availability query resolves async)
			await waitFor(() => {
				const priceCell = screen.getByTestId("price-product-1");
				expect(priceCell.textContent).toBe("-");
			});
		});

		it("should display JPY currency without decimals", async () => {
			const product = createMockProduct({ id: "product-1", name: "Test" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 1500,
				today_average_price_minor_units: null,
				yesterday_average_price_minor_units: null,
				price_currency: "JPY",
				currency_exponent: 0,
			});

			getMockedInvoke().mockImplementation((cmd: string, args?: InvokeArgs) => {
				const typedArgs = args as { productId?: string } | undefined;

				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (
					cmd === COMMANDS.GET_LATEST_AVAILABILITY &&
					typedArgs?.productId === "product-1"
				) {
					return Promise.resolve(check);
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Test")).toBeInTheDocument();
			});

			// Wait for JPY price to appear (availability query resolves async)
			await waitFor(() => {
				expect(screen.getByText("¥1,500")).toBeInTheDocument();
			});
		});

		it("should display zero price", async () => {
			const product = createMockProduct({ id: "product-1", name: "Test" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 0,
				today_average_price_minor_units: null,
				yesterday_average_price_minor_units: null,
				price_currency: "USD",
				currency_exponent: 2,
			});

			getMockedInvoke().mockImplementation((cmd: string, args?: InvokeArgs) => {
				const typedArgs = args as { productId?: string } | undefined;

				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (
					cmd === COMMANDS.GET_LATEST_AVAILABILITY &&
					typedArgs?.productId === "product-1"
				) {
					return Promise.resolve(check);
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Test")).toBeInTheDocument();
			});

			// Wait for zero price to appear (availability query resolves async)
			await waitFor(() => {
				expect(screen.getByText("$0.00")).toBeInTheDocument();
			});

			// Assert no trend indicators
			expect(screen.queryByText(/-\d+%/)).not.toBeInTheDocument();
			expect(screen.queryByText(/\+\d+%/)).not.toBeInTheDocument();
		});

		it("should display price without trend indicator when unchanged", async () => {
			const product = createMockProduct({ id: "product-1", name: "Test" });
			const check = createMockAvailabilityCheck({
				product_id: "product-1",
				price_minor_units: 79900,
				today_average_price_minor_units: 79900,
				yesterday_average_price_minor_units: 79900,
				price_currency: "USD",
				currency_exponent: 2,
			});

			getMockedInvoke().mockImplementation((cmd: string, args?: InvokeArgs) => {
				const typedArgs = args as { productId?: string } | undefined;

				if (cmd === COMMANDS.GET_PRODUCTS) return Promise.resolve([product]);
				if (
					cmd === COMMANDS.GET_LATEST_AVAILABILITY &&
					typedArgs?.productId === "product-1"
				) {
					return Promise.resolve(check);
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Test")).toBeInTheDocument();
			});

			// Wait for price to appear (availability query resolves async)
			await waitFor(() => {
				expect(screen.getByText("$799.00")).toBeInTheDocument();
			});

			// Assert no trend indicators (0% change = no comparison shown)
			expect(screen.queryByText(/-\d+%/)).not.toBeInTheDocument();
			expect(screen.queryByText(/\+\d+%/)).not.toBeInTheDocument();
		});
	});
});
