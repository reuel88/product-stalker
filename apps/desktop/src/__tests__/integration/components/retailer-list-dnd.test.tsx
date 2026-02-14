import { describe, expect, it, vi } from "vitest";
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
		retailer_id: "retailer-1",
		url: "https://www.amazon.com/dp/B123",
		label: null,
		sort_order: 0,
		created_at: new Date().toISOString(),
		...overrides,
	};
}

describe("RetailerList DnD", () => {
	const mockOnRemove = vi.fn();
	const mockOnReorder = vi.fn();

	it("should render drag handles for all retailers", () => {
		const retailers = [
			createRetailer({ id: "pr-1", sort_order: 0 }),
			createRetailer({
				id: "pr-2",
				url: "https://www.walmart.com/item/456",
				sort_order: 1,
			}),
			createRetailer({
				id: "pr-3",
				url: "https://www.bestbuy.com/product/789",
				sort_order: 2,
			}),
		];

		render(
			<RetailerList
				retailers={retailers}
				onRemove={mockOnRemove}
				isRemoving={false}
				onReorder={mockOnReorder}
			/>,
		);

		expect(screen.getByTestId("drag-handle-pr-1")).toBeInTheDocument();
		expect(screen.getByTestId("drag-handle-pr-2")).toBeInTheDocument();
		expect(screen.getByTestId("drag-handle-pr-3")).toBeInTheDocument();
	});

	it("should not render drag handles when list is empty", () => {
		render(
			<RetailerList
				retailers={[]}
				onRemove={mockOnRemove}
				isRemoving={false}
				onReorder={mockOnReorder}
			/>,
		);

		expect(screen.queryByTestId(/drag-handle-/)).not.toBeInTheDocument();
		expect(screen.getByText(/No retailers added yet/)).toBeInTheDocument();
	});

	it("should have drag handle with correct aria-label", () => {
		const retailers = [createRetailer({ id: "pr-1" })];

		render(
			<RetailerList
				retailers={retailers}
				onRemove={mockOnRemove}
				isRemoving={false}
				onReorder={mockOnReorder}
			/>,
		);

		const handle = screen.getByTestId("drag-handle-pr-1");
		expect(handle).toHaveAttribute("aria-label", "Drag to reorder");
	});
});
