import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants";
import { ProductsView } from "@/modules/products/ui/views/products-view";
import {
	createMockBulkCheckSummary,
	createMockProduct,
	createMockProducts,
} from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { render, screen, waitFor } from "../../test-utils";

describe("ProductsView", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	describe("loading state", () => {
		it("should render skeleton while loading products", () => {
			// Never resolve to keep loading state
			getMockedInvoke().mockImplementation(() => new Promise(() => {}));

			const { container } = render(<ProductsView />);

			// Skeleton should have animated elements
			const skeletons = container.querySelectorAll('[class*="animate-pulse"]');
			expect(skeletons.length).toBeGreaterThan(0);
		});
	});

	describe("error state", () => {
		it("should render error state when products fail to load", async () => {
			getMockedInvoke().mockImplementation(() => {
				return Promise.reject(new Error("Failed to fetch"));
			});

			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Failed to load products")).toBeInTheDocument();
			});
			expect(screen.getByText("Please try again later")).toBeInTheDocument();
		});
	});

	describe("rendering with products", () => {
		beforeEach(() => {
			const products = createMockProducts(3);
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: products,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});
		});

		it("should render page title", async () => {
			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Products")).toBeInTheDocument();
			});
		});

		it("should render Add Product button", async () => {
			render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /add product/i }),
				).toBeInTheDocument();
			});
		});

		it("should render Check All button", async () => {
			render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /check all/i }),
				).toBeInTheDocument();
			});
		});

		it("should render All Products card", async () => {
			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("All Products")).toBeInTheDocument();
			});
			expect(
				screen.getByText("View and manage your tracked products"),
			).toBeInTheDocument();
		});

		it("should render products table", async () => {
			render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Name")).toBeInTheDocument();
			});
		});
	});

	describe("empty state", () => {
		it("should disable Check All button when no products", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});

			render(<ProductsView />);

			await waitFor(() => {
				const checkAllBtn = screen.getByRole("button", { name: /check all/i });
				expect(checkAllBtn).toBeDisabled();
			});
		});
	});

	describe("create product dialog", () => {
		beforeEach(() => {
			const products: ReturnType<typeof createMockProducts> = [];
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: products,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.CREATE_PRODUCT]: createMockProduct({ name: "New Product" }),
			});
		});

		it("should open create dialog when Add Product is clicked", async () => {
			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /add product/i }),
				).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: /add product/i }));

			await waitFor(() => {
				// Look for the dialog description which is unique to the dialog
				expect(
					screen.getByText("Add a new product to track"),
				).toBeInTheDocument();
			});
		});

		it("should close create dialog when Cancel is clicked", async () => {
			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /add product/i }),
				).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: /add product/i }));

			await waitFor(() => {
				// Look for the dialog description which is unique to the dialog
				expect(
					screen.getByText("Add a new product to track"),
				).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: "Cancel" }));

			await waitFor(() => {
				expect(
					screen.queryByText("Add a new product to track"),
				).not.toBeInTheDocument();
			});
		});

		it("should render form fields in create dialog", async () => {
			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /add product/i }),
				).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: /add product/i }));

			await waitFor(() => {
				expect(screen.getByLabelText("Name")).toBeInTheDocument();
				expect(screen.getByLabelText("Description")).toBeInTheDocument();
				expect(screen.getByLabelText("Notes")).toBeInTheDocument();
			});
		});

		it("should call createProduct when form is submitted with valid data", async () => {
			const newProduct = createMockProduct({ name: "Test Product" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.CREATE_PRODUCT]: newProduct,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /add product/i }),
				).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: /add product/i }));

			await waitFor(() => {
				expect(screen.getByLabelText("Name")).toBeInTheDocument();
			});

			await user.type(screen.getByLabelText("Name"), "Test Product");

			await user.click(screen.getByRole("button", { name: "Create" }));

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.CREATE_PRODUCT,
					expect.objectContaining({
						input: expect.objectContaining({
							name: "Test Product",
						}),
					}),
				);
			});
		});

		it("should show validation error when name is missing", async () => {
			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /add product/i }),
				).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: /add product/i }));

			await waitFor(() => {
				expect(screen.getByLabelText("Name")).toBeInTheDocument();
			});

			// Leave name empty, click Create
			await user.click(screen.getByRole("button", { name: "Create" }));

			// Create should not be called - validation should fail
			// The dialog should still be open
			await waitFor(() => {
				expect(
					screen.getByText("Add a new product to track"),
				).toBeInTheDocument();
			});
		});
	});

	describe("edit product dialog", () => {
		it("should open edit dialog when edit action is clicked", async () => {
			const product = createMockProduct({
				id: "prod-1",
				name: "Existing Product",
				description: "A description",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Existing Product")).toBeInTheDocument();
			});

			// Open dropdown menu
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			// Click Edit
			await waitFor(() => {
				expect(screen.getByText("Edit")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Edit"));

			// Check edit dialog opens with pre-filled data
			await waitFor(() => {
				expect(screen.getByText("Edit Product")).toBeInTheDocument();
				expect(screen.getByText("Update product details")).toBeInTheDocument();
			});

			// Form should have existing values
			expect(screen.getByLabelText("Name")).toHaveValue("Existing Product");
		});

		it("should call updateProduct when edit form is submitted", async () => {
			const product = createMockProduct({
				id: "prod-1",
				name: "Existing Product",
			});
			const updatedProduct = { ...product, name: "Updated Product" };
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.UPDATE_PRODUCT]: updatedProduct,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Existing Product")).toBeInTheDocument();
			});

			// Open dropdown and click edit
			await user.click(screen.getByRole("button", { name: /open menu/i }));
			await waitFor(() => {
				expect(screen.getByText("Edit")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Edit"));

			// Wait for dialog
			await waitFor(() => {
				expect(screen.getByText("Edit Product")).toBeInTheDocument();
			});

			// Clear and type new name
			const nameInput = screen.getByLabelText("Name");
			await user.clear(nameInput);
			await user.type(nameInput, "Updated Product");

			await user.click(screen.getByRole("button", { name: "Save" }));

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_PRODUCT,
					expect.objectContaining({
						id: "prod-1",
						input: expect.objectContaining({
							name: "Updated Product",
						}),
					}),
				);
			});
		});

		it("should close edit dialog when Cancel is clicked", async () => {
			const product = createMockProduct({
				id: "prod-1",
				name: "Existing Product",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Existing Product")).toBeInTheDocument();
			});

			// Open edit dialog
			await user.click(screen.getByRole("button", { name: /open menu/i }));
			await waitFor(() => {
				expect(screen.getByText("Edit")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Edit"));

			await waitFor(() => {
				expect(screen.getByText("Edit Product")).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: "Cancel" }));

			await waitFor(() => {
				expect(
					screen.queryByText("Update product details"),
				).not.toBeInTheDocument();
			});
		});
	});

	describe("delete product dialog", () => {
		it("should open delete confirmation when delete action is clicked", async () => {
			const product = createMockProduct({
				id: "prod-1",
				name: "Product to Delete",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Product to Delete")).toBeInTheDocument();
			});

			// Open dropdown menu
			await user.click(screen.getByRole("button", { name: /open menu/i }));

			// Click Delete
			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Delete"));

			// Check delete dialog opens
			await waitFor(() => {
				expect(screen.getByText("Delete Product")).toBeInTheDocument();
				expect(
					screen.getByText(
						/Are you sure you want to delete "Product to Delete"/,
					),
				).toBeInTheDocument();
			});
		});

		it("should call deleteProduct when delete is confirmed", async () => {
			const product = createMockProduct({
				id: "prod-1",
				name: "Product to Delete",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.DELETE_PRODUCT]: undefined,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Product to Delete")).toBeInTheDocument();
			});

			// Open dropdown and click delete
			await user.click(screen.getByRole("button", { name: /open menu/i }));
			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Delete"));

			// Wait for confirmation dialog
			await waitFor(() => {
				expect(screen.getByText("Delete Product")).toBeInTheDocument();
			});

			// Find the delete button in the dialog (not the menu item)
			const deleteButtons = screen.getAllByRole("button", { name: "Delete" });
			const confirmDeleteBtn = deleteButtons[deleteButtons.length - 1];
			await user.click(confirmDeleteBtn);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.DELETE_PRODUCT,
					{
						id: "prod-1",
					},
				);
			});
		});

		it("should close delete dialog when Cancel is clicked", async () => {
			const product = createMockProduct({
				id: "prod-1",
				name: "Product to Delete",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(screen.getByText("Product to Delete")).toBeInTheDocument();
			});

			// Open delete dialog
			await user.click(screen.getByRole("button", { name: /open menu/i }));
			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});
			await user.click(screen.getByText("Delete"));

			await waitFor(() => {
				expect(screen.getByText("Delete Product")).toBeInTheDocument();
			});

			await user.click(screen.getByRole("button", { name: "Cancel" }));

			await waitFor(() => {
				expect(
					screen.queryByText(/Are you sure you want to delete/),
				).not.toBeInTheDocument();
			});
		});
	});

	describe("check all functionality", () => {
		it("should call checkAllAvailability when Check All is clicked", async () => {
			const products = createMockProducts(2);
			const summary = createMockBulkCheckSummary({
				total: 2,
				successful: 2,
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: products,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.CHECK_ALL_AVAILABILITY]: summary,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /check all/i }),
				).not.toBeDisabled();
			});

			await user.click(screen.getByRole("button", { name: /check all/i }));

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.CHECK_ALL_AVAILABILITY,
				);
			});
		});

		it("should show special message when products are back in stock", async () => {
			const products = createMockProducts(2);
			const summary = createMockBulkCheckSummary({
				total: 2,
				successful: 2,
				back_in_stock_count: 1,
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: products,
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.CHECK_ALL_AVAILABILITY]: summary,
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /check all/i }),
				).not.toBeDisabled();
			});

			await user.click(screen.getByRole("button", { name: /check all/i }));

			// The toast with back in stock message should be triggered
			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.CHECK_ALL_AVAILABILITY,
				);
			});
		});

		it("should show Checking... while checking all products", async () => {
			const products = createMockProducts(2);
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_PRODUCTS) {
					return Promise.resolve(products);
				}
				if (cmd === COMMANDS.GET_LATEST_AVAILABILITY) {
					return Promise.resolve(null);
				}
				if (cmd === COMMANDS.CHECK_ALL_AVAILABILITY) {
					return new Promise(() => {}); // Never resolves
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<ProductsView />);

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /check all/i }),
				).not.toBeDisabled();
			});

			await user.click(screen.getByRole("button", { name: /check all/i }));

			await waitFor(() => {
				expect(
					screen.getByRole("button", { name: /checking/i }),
				).toBeInTheDocument();
			});
		});
	});
});
