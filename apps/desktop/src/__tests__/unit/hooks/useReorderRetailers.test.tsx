import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS, QUERY_KEYS } from "@/constants";
import { useReorderRetailers } from "@/modules/products/hooks/useReorderRetailers";
import type { ProductRetailerResponse } from "@/modules/products/types";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { createHookWrapper, createTestQueryClient } from "../../test-utils";

function createRetailer(
	overrides: Partial<ProductRetailerResponse> = {},
): ProductRetailerResponse {
	return {
		id: "pr-1",
		product_id: "product-1",
		retailer_id: "retailer-1",
		url: "https://amazon.com/dp/B123",
		label: null,
		sort_order: 0,
		created_at: new Date().toISOString(),
		...overrides,
	};
}

const PRODUCT_ID = "product-1";

describe("useReorderRetailers", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	it("should invoke reorder_product_retailers with correct payload", async () => {
		mockInvokeMultiple({
			[COMMANDS.REORDER_PRODUCT_RETAILERS]: undefined,
			[COMMANDS.GET_PRODUCT_RETAILERS]: [],
		});

		const { result } = renderHook(() => useReorderRetailers(PRODUCT_ID), {
			wrapper: createHookWrapper(),
		});

		const retailers = [
			createRetailer({ id: "pr-a", sort_order: 0 }),
			createRetailer({ id: "pr-b", sort_order: 1 }),
			createRetailer({ id: "pr-c", sort_order: 2 }),
		];

		// Reorder: move C to front
		const newOrder = [retailers[2], retailers[0], retailers[1]];

		await act(async () => {
			result.current.mutate(newOrder);
		});

		await waitFor(() => {
			expect(result.current.isSuccess).toBe(true);
		});

		const mockedInvoke = getMockedInvoke();
		const reorderCall = mockedInvoke.mock.calls.find(
			(call) => call[0] === COMMANDS.REORDER_PRODUCT_RETAILERS,
		);
		expect(reorderCall).toBeDefined();
		expect(reorderCall?.[1]).toEqual({
			input: {
				updates: [
					{ id: "pr-c", sort_order: 0 },
					{ id: "pr-a", sort_order: 1 },
					{ id: "pr-b", sort_order: 2 },
				],
			},
		});
	});

	it("should optimistically update cache on mutate", async () => {
		const queryClient = createTestQueryClient();

		const initialRetailers: ProductRetailerResponse[] = [
			createRetailer({ id: "pr-a", sort_order: 0 }),
			createRetailer({ id: "pr-b", sort_order: 1 }),
		];

		queryClient.setQueryData(
			QUERY_KEYS.productRetailers(PRODUCT_ID),
			initialRetailers,
		);

		// Never resolve the reorder command so onSettled never fires
		// and we can inspect the optimistic state set by onMutate
		getMockedInvoke().mockImplementation((cmd: string) => {
			if (cmd === COMMANDS.REORDER_PRODUCT_RETAILERS) {
				return new Promise(() => {});
			}
			return Promise.resolve(initialRetailers);
		});

		function Wrapper({ children }: { children: ReactNode }) {
			return (
				<QueryClientProvider client={queryClient}>
					{children}
				</QueryClientProvider>
			);
		}

		const { result } = renderHook(() => useReorderRetailers(PRODUCT_ID), {
			wrapper: Wrapper,
		});

		// Reverse order
		const newOrder = [initialRetailers[1], initialRetailers[0]];

		await act(async () => {
			result.current.mutate(newOrder);
		});

		// Check cache was updated optimistically
		await waitFor(() => {
			const cached = queryClient.getQueryData<ProductRetailerResponse[]>(
				QUERY_KEYS.productRetailers(PRODUCT_ID),
			);
			expect(cached?.[0].id).toBe("pr-b");
			expect(cached?.[0].sort_order).toBe(0);
			expect(cached?.[1].id).toBe("pr-a");
			expect(cached?.[1].sort_order).toBe(1);
		});
	});
});
