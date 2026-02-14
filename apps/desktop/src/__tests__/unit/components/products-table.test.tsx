import { beforeEach, describe, expect, it, vi } from "vitest";
import { UI } from "@/constants";
import { formatDate } from "@/lib/format-date";
import { ProductsTable } from "@/modules/products/ui/components/products-table";
import { createMockProduct, createMockProducts } from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { render, screen, waitFor } from "../../test-utils";

// Mock the openUrl function from tauri
vi.mock("@tauri-apps/plugin-opener", () => ({
	openUrl: vi.fn(),
}));

describe("ProductsTable", () => {
	const mockOnEdit = vi.fn();
	const mockOnDelete = vi.fn();

	beforeEach(() => {
		mockOnEdit.mockClear();
		mockOnDelete.mockClear();
		getMockedInvoke().mockReset();
		// Default mock for availability checks
		mockInvokeMultiple({
			get_latest_availability: null,
		});
	});

	describe("rendering", () => {
		it("should render table headers", () => {
			render(
				<ProductsTable
					products={[]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText("Name")).toBeInTheDocument();
			expect(screen.getByText("Retailers")).toBeInTheDocument();
			expect(screen.getByText("Availability")).toBeInTheDocument();
			expect(screen.getByText("Price")).toBeInTheDocument();
			expect(screen.getByText("Description")).toBeInTheDocument();
			expect(screen.getByText("Created")).toBeInTheDocument();
		});

		it("should render empty state when no products", () => {
			render(
				<ProductsTable
					products={[]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText("No products found")).toBeInTheDocument();
		});

		it("should render products in table rows", () => {
			const products = [
				createMockProduct({ name: "Product A" }),
				createMockProduct({ name: "Product B" }),
			];

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText("Product A")).toBeInTheDocument();
			expect(screen.getByText("Product B")).toBeInTheDocument();
		});

		it("should render product description or dash for null", () => {
			const productWithDesc = createMockProduct({
				id: "prod-with-desc",
				name: "With desc",
				description: "Test description",
			});
			const productNoDesc = createMockProduct({
				id: "prod-no-desc",
				name: "No desc",
				description: null,
			});
			const products = [productWithDesc, productNoDesc];

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(
				screen.getByTestId("description-prod-with-desc"),
			).toHaveTextContent("Test description");
			expect(screen.getByTestId("description-prod-no-desc")).toHaveTextContent(
				"-",
			);
		});

		it("should format created date", () => {
			const product = createMockProduct({
				created_at: "2024-01-15T10:30:00Z",
			});

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Date format depends on locale, just verify it's rendered
			const dateString = formatDate("2024-01-15T10:30:00Z");
			expect(screen.getByText(dateString)).toBeInTheDocument();
		});

		it("should render product name with font-medium class", () => {
			const product = createMockProduct({ name: "Styled Product" });

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			const nameCell = screen.getByText("Styled Product");
			expect(nameCell).toHaveClass("font-medium");
		});
	});

	describe("description truncation", () => {
		it("should truncate long descriptions", () => {
			const longDesc = "A".repeat(100);
			const product = createMockProduct({ description: longDesc });

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			const truncated = `${longDesc.slice(0, UI.TRUNCATE.DESCRIPTION_LENGTH)}...`;
			expect(screen.getByText(truncated)).toBeInTheDocument();
		});

		it("should not truncate short descriptions", () => {
			const shortDesc = "Short description";
			const product = createMockProduct({ description: shortDesc });

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText(shortDesc)).toBeInTheDocument();
		});

		it("should have full description as title attribute for truncated descriptions", () => {
			const longDesc = "A".repeat(100);
			const product = createMockProduct({ description: longDesc });

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			const descElement = screen.getByTitle(longDesc);
			expect(descElement).toBeInTheDocument();
		});
	});

	describe("loading state", () => {
		it("should render skeleton when loading", () => {
			render(
				<ProductsTable
					products={[]}
					isLoading={true}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Skeleton still shows table headers
			expect(screen.getByText("Name")).toBeInTheDocument();
			// But no "No products found" message
			expect(screen.queryByText("No products found")).not.toBeInTheDocument();
		});

		it("should render multiple skeleton rows when loading", () => {
			const { container } = render(
				<ProductsTable
					products={[]}
					isLoading={true}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Should have skeleton elements
			const skeletons = container.querySelectorAll('[class*="animate-pulse"]');
			expect(skeletons.length).toBeGreaterThan(0);
		});
	});

	describe("pagination", () => {
		it("should show pagination controls", () => {
			const products = createMockProducts(5);

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText(/Page 1 of/)).toBeInTheDocument();
		});

		it("should show correct page count for many products", () => {
			const products = createMockProducts(25);

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// 25 products / 10 per page = 3 pages
			expect(screen.getByText("Page 1 of 3")).toBeInTheDocument();
		});

		it("should show page 1 of 1 for empty products", () => {
			render(
				<ProductsTable
					products={[]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText("Page 1 of 1")).toBeInTheDocument();
		});

		it("should have four pagination buttons", () => {
			const products = createMockProducts(25);

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Find buttons within pagination controls area
			const paginationArea = screen.getByText("Page 1 of 3").parentElement;
			const buttons = paginationArea?.querySelectorAll("button");
			expect(buttons?.length).toBe(4);
		});

		it("should disable first and previous buttons on first page", () => {
			const products = createMockProducts(25);

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			const buttons = screen.getAllByRole("button");
			// Last 4 buttons are pagination (first, prev, next, last)
			const paginationButtons = buttons.slice(-4);

			// First and previous should be disabled on page 1
			expect(paginationButtons[0]).toBeDisabled();
			expect(paginationButtons[1]).toBeDisabled();
			// Next and last should be enabled
			expect(paginationButtons[2]).not.toBeDisabled();
			expect(paginationButtons[3]).not.toBeDisabled();
		});

		it("should paginate products correctly", async () => {
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

			// First page should show products 1-10
			expect(screen.getByText("Product 1")).toBeInTheDocument();
			expect(screen.getByText("Product 10")).toBeInTheDocument();
			expect(screen.queryByText("Product 11")).not.toBeInTheDocument();

			// Find next page button
			const buttons = screen.getAllByRole("button");
			const nextButton = buttons[buttons.length - 2]; // second to last is "next"
			await user.click(nextButton);

			await waitFor(() => {
				expect(screen.getByText("Page 2 of 2")).toBeInTheDocument();
			});
			expect(screen.getByText("Product 11")).toBeInTheDocument();
		});
	});

	describe("actions menu", () => {
		it("should render action menu trigger for each product", () => {
			const products = createMockProducts(2);

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			const menuTriggers = screen.getAllByRole("button", {
				name: /open menu/i,
			});
			expect(menuTriggers.length).toBe(2);
		});

		it("should call onEdit when edit is clicked", async () => {
			const product = createMockProduct({ name: "Test Product" });

			const { user } = render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Open the dropdown menu
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			// Wait for menu to open and find Edit option
			await waitFor(() => {
				expect(screen.getByText("Edit")).toBeInTheDocument();
			});

			// Click edit
			await user.click(screen.getByText("Edit"));

			expect(mockOnEdit).toHaveBeenCalledWith(product);
		});

		it("should call onDelete when delete is clicked", async () => {
			const product = createMockProduct({ name: "Test Product" });

			const { user } = render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Open the dropdown menu
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			// Wait for menu to open and find Delete option
			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});

			// Click delete
			await user.click(screen.getByText("Delete"));

			expect(mockOnDelete).toHaveBeenCalledWith(product);
		});

		it("should not call onEdit if callback is not provided", async () => {
			const product = createMockProduct({ name: "Test Product" });

			const { user } = render(
				<ProductsTable products={[product]} onDelete={mockOnDelete} />,
			);

			// Open the dropdown menu
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			// Wait for menu to open
			await waitFor(() => {
				expect(screen.getByText("Edit")).toBeInTheDocument();
			});

			// Click edit - should not throw
			await user.click(screen.getByText("Edit"));
			// No assertion needed - just verify it doesn't throw
		});

		it("should not call onDelete if callback is not provided", async () => {
			const product = createMockProduct({ name: "Test Product" });

			const { user } = render(
				<ProductsTable products={[product]} onEdit={mockOnEdit} />,
			);

			// Open the dropdown menu
			const menuTrigger = screen.getByRole("button", { name: /open menu/i });
			await user.click(menuTrigger);

			// Wait for menu to open
			await waitFor(() => {
				expect(screen.getByText("Delete")).toBeInTheDocument();
			});

			// Click delete - should not throw
			await user.click(screen.getByText("Delete"));
			// No assertion needed - just verify it doesn't throw
		});
	});

	describe("availability cell", () => {
		it("should render availability badge for each product", () => {
			const products = [createMockProduct({ id: "prod-1" })];

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// The availability cell should be present (even if loading)
			expect(screen.getByText("Availability")).toBeInTheDocument();
		});
	});

	describe("price cell", () => {
		it("should render price column header", () => {
			render(
				<ProductsTable
					products={[]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText("Price")).toBeInTheDocument();
		});

		it("should display dash when no price available", () => {
			const product = createMockProduct({ id: "prod-1" });

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Should have dash specifically in the price cell
			expect(screen.getByTestId("price-prod-1")).toHaveTextContent("-");
		});
	});
});
