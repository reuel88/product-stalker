import { beforeEach, describe, expect, it, vi } from "vitest";
import { ProductsTable } from "@/modules/products/ui/components/products-table";
import { createMockProduct, createMockProducts } from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { render, screen, waitFor } from "../../test-utils";

vi.mock("@tauri-apps/plugin-opener", () => ({
	openUrl: vi.fn(),
}));

describe("ProductsTable DnD reorder mode", () => {
	const mockOnEdit = vi.fn();
	const mockOnDelete = vi.fn();

	beforeEach(() => {
		mockOnEdit.mockClear();
		mockOnDelete.mockClear();
		getMockedInvoke().mockReset();
		mockInvokeMultiple({
			get_latest_availability: null,
			get_product_retailers: [],
			reorder_products: undefined,
		});
	});

	it("should render reorder toggle button", () => {
		render(
			<ProductsTable
				products={createMockProducts(3)}
				onEdit={mockOnEdit}
				onDelete={mockOnDelete}
			/>,
		);

		expect(screen.getByTestId("reorder-toggle")).toBeInTheDocument();
		expect(screen.getByTestId("reorder-toggle")).toHaveTextContent("Reorder");
	});

	it("should toggle to reorder mode and show drag handles", async () => {
		const products = [
			createMockProduct({ id: "prod-1" }),
			createMockProduct({ id: "prod-2" }),
		];

		const { user } = render(
			<ProductsTable
				products={products}
				onEdit={mockOnEdit}
				onDelete={mockOnDelete}
			/>,
		);

		// Initially no drag handles
		expect(screen.queryByTestId("drag-handle-prod-1")).not.toBeInTheDocument();

		// Click reorder toggle
		await user.click(screen.getByTestId("reorder-toggle"));

		// Now drag handles should be visible
		expect(screen.getByTestId("drag-handle-prod-1")).toBeInTheDocument();
		expect(screen.getByTestId("drag-handle-prod-2")).toBeInTheDocument();

		// Button should say "Done"
		expect(screen.getByTestId("reorder-toggle")).toHaveTextContent("Done");
	});

	it("should hide pagination in reorder mode", async () => {
		const products = createMockProducts(15).map((p, i) => ({
			...p,
			name: `Product ${i + 1}`,
		}));

		const { user } = render(
			<ProductsTable
				products={products}
				onEdit={mockOnEdit}
				onDelete={mockOnDelete}
			/>,
		);

		// Pagination visible in normal mode
		expect(screen.getByText(/Page 1 of/)).toBeInTheDocument();

		// Enter reorder mode
		await user.click(screen.getByTestId("reorder-toggle"));

		// Pagination should be hidden
		expect(screen.queryByText(/Page \d+ of/)).not.toBeInTheDocument();
	});

	it("should show all products in reorder mode (no pagination)", async () => {
		const products = createMockProducts(15).map((p, i) => ({
			...p,
			name: `Product ${i + 1}`,
		}));

		const { user } = render(
			<ProductsTable
				products={products}
				onEdit={mockOnEdit}
				onDelete={mockOnDelete}
			/>,
		);

		// In normal mode, only first page visible
		expect(screen.queryByText("Product 11")).not.toBeInTheDocument();

		// Enter reorder mode
		await user.click(screen.getByTestId("reorder-toggle"));

		// All products should now be visible
		await waitFor(() => {
			expect(screen.getByText("Product 11")).toBeInTheDocument();
		});
		expect(screen.getByText("Product 15")).toBeInTheDocument();
	});

	it("should show all products when entering reorder mode from page 2", async () => {
		const products = createMockProducts(15).map((p, i) => ({
			...p,
			name: `Product ${i + 1}`,
		}));

		const { user } = render(
			<ProductsTable
				products={products}
				onEdit={mockOnEdit}
				onDelete={mockOnDelete}
			/>,
		);

		// Confirm we're on page 1 of 2
		expect(screen.getByText("Page 1 of 2")).toBeInTheDocument();

		// Find the next page button — the 3rd icon-sized button in the pagination row
		const paginationButtons = screen
			.getByText("Page 1 of 2")
			.closest("div")!
			.parentElement!.querySelectorAll("button");
		const nextPageButton = paginationButtons[2];
		await user.click(nextPageButton);

		// Confirm we're on page 2
		expect(screen.getByText("Page 2 of 2")).toBeInTheDocument();
		// Page 2 should not have Product 1
		expect(screen.queryByText("Product 1")).not.toBeInTheDocument();

		// Enter reorder mode from page 2
		await user.click(screen.getByTestId("reorder-toggle"));

		// All products should be visible (not an empty page)
		await waitFor(() => {
			expect(screen.getByText("Product 1")).toBeInTheDocument();
		});
		expect(screen.getByText("Product 15")).toBeInTheDocument();
	});

	it("should hide drag handles when exiting reorder mode", async () => {
		const products = [createMockProduct({ id: "prod-1" })];

		const { user } = render(
			<ProductsTable
				products={products}
				onEdit={mockOnEdit}
				onDelete={mockOnDelete}
			/>,
		);

		// Enter reorder mode
		await user.click(screen.getByTestId("reorder-toggle"));
		expect(screen.getByTestId("drag-handle-prod-1")).toBeInTheDocument();

		// Exit reorder mode
		await user.click(screen.getByTestId("reorder-toggle"));
		expect(screen.queryByTestId("drag-handle-prod-1")).not.toBeInTheDocument();
	});

	it("should restore pagination when exiting reorder mode", async () => {
		const products = createMockProducts(15);

		const { user } = render(
			<ProductsTable
				products={products}
				onEdit={mockOnEdit}
				onDelete={mockOnDelete}
			/>,
		);

		// Enter reorder mode
		await user.click(screen.getByTestId("reorder-toggle"));
		expect(screen.queryByText(/Page \d+ of/)).not.toBeInTheDocument();

		// Exit reorder mode
		await user.click(screen.getByTestId("reorder-toggle"));
		expect(screen.getByText(/Page 1 of/)).toBeInTheDocument();
	});
});
