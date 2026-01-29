import { beforeEach, describe, expect, it, vi } from "vitest";
import { ProductsTable } from "@/components/products-table";
import { UI } from "@/constants";
import { createMockProduct, createMockProducts } from "../../mocks/data";
import { render, screen, waitFor } from "../../test-utils";

describe("ProductsTable", () => {
	const mockOnEdit = vi.fn();
	const mockOnDelete = vi.fn();

	beforeEach(() => {
		mockOnEdit.mockClear();
		mockOnDelete.mockClear();
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
			expect(screen.getByText("URL")).toBeInTheDocument();
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
				createMockProduct({ name: "Product A", url: "https://example.com/a" }),
				createMockProduct({ name: "Product B", url: "https://example.com/b" }),
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
			const products = [
				createMockProduct({
					name: "With desc",
					description: "Test description",
				}),
				createMockProduct({ name: "No desc", description: null }),
			];

			render(
				<ProductsTable
					products={products}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText("Test description")).toBeInTheDocument();
			expect(screen.getByText("-")).toBeInTheDocument();
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
			const dateString = new Date("2024-01-15T10:30:00Z").toLocaleDateString();
			expect(screen.getByText(dateString)).toBeInTheDocument();
		});
	});

	describe("URL truncation", () => {
		it("should truncate long URLs", () => {
			const longUrl =
				"https://example.com/very/long/path/that/exceeds/the/limit/and/should/be/truncated";
			const product = createMockProduct({ url: longUrl });

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			const truncated = `${longUrl.slice(0, UI.TRUNCATE.URL_LENGTH)}...`;
			expect(screen.getByText(truncated)).toBeInTheDocument();
		});

		it("should not truncate short URLs", () => {
			const shortUrl = "https://example.com";
			const product = createMockProduct({ url: shortUrl });

			render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			expect(screen.getByText(shortUrl)).toBeInTheDocument();
		});

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

			// Get all pagination buttons (first, prev, next, last)
			const paginationButtons = screen
				.getAllByRole("button")
				.filter((btn) => btn.closest(".flex.items-center.gap-1"));

			// Find next page button by looking for one that's not disabled and navigates forward
			const buttons = screen.getAllByRole("button");
			// Buttons at end are pagination: first, prev, next, last
			const nextButton = buttons[buttons.length - 2]; // second to last is "next"
			await user.click(nextButton);

			await waitFor(() => {
				expect(screen.getByText("Page 2 of 2")).toBeInTheDocument();
			});
			expect(screen.getByText("Product 11")).toBeInTheDocument();
		});
	});

	describe("actions", () => {
		it("should call onEdit when edit is clicked", async () => {
			const product = createMockProduct({ name: "Test Product" });

			const { user } = render(
				<ProductsTable
					products={[product]}
					onEdit={mockOnEdit}
					onDelete={mockOnDelete}
				/>,
			);

			// Open the dropdown menu - find by sr-only text
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
	});
});
