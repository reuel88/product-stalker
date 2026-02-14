import { QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS, QUERY_KEYS } from "@/constants";
import { useReorderProducts } from "@/modules/products/hooks/useReorderProducts";
import type { ProductResponse } from "@/modules/products/types";
import { createMockProduct } from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { createHookWrapper, createTestQueryClient } from "../../test-utils";

describe("useReorderProducts", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	it("should invoke reorder_products with correct payload", async () => {
		mockInvokeMultiple({
			[COMMANDS.REORDER_PRODUCTS]: undefined,
			[COMMANDS.GET_PRODUCTS]: [],
		});

		const { result } = renderHook(() => useReorderProducts(), {
			wrapper: createHookWrapper(),
		});

		const products = [
			createMockProduct({ id: "prod-a", sort_order: 0 }),
			createMockProduct({ id: "prod-b", sort_order: 1 }),
			createMockProduct({ id: "prod-c", sort_order: 2 }),
		];

		// Reorder: move C to front
		const newOrder = [products[2], products[0], products[1]];

		await act(async () => {
			result.current.mutate(newOrder);
		});

		await waitFor(() => {
			expect(result.current.isSuccess).toBe(true);
		});

		const mockedInvoke = getMockedInvoke();
		const reorderCall = mockedInvoke.mock.calls.find(
			(call) => call[0] === COMMANDS.REORDER_PRODUCTS,
		);
		expect(reorderCall).toBeDefined();
		expect(reorderCall?.[1]).toEqual({
			input: {
				updates: [
					{ id: "prod-c", sort_order: 0 },
					{ id: "prod-a", sort_order: 1 },
					{ id: "prod-b", sort_order: 2 },
				],
			},
		});
	});

	it("should optimistically update cache on mutate", async () => {
		const queryClient = createTestQueryClient();

		const initialProducts: ProductResponse[] = [
			createMockProduct({ id: "prod-a", name: "A", sort_order: 0 }),
			createMockProduct({ id: "prod-b", name: "B", sort_order: 1 }),
		];

		queryClient.setQueryData(QUERY_KEYS.PRODUCTS, initialProducts);

		// Never resolve the reorder command so onSettled never fires
		// and we can inspect the optimistic state set by onMutate
		getMockedInvoke().mockImplementation((cmd: string) => {
			if (cmd === COMMANDS.REORDER_PRODUCTS) {
				return new Promise(() => {});
			}
			return Promise.resolve(initialProducts);
		});

		function Wrapper({ children }: { children: ReactNode }) {
			return (
				<QueryClientProvider client={queryClient}>
					{children}
				</QueryClientProvider>
			);
		}

		const { result } = renderHook(() => useReorderProducts(), {
			wrapper: Wrapper,
		});

		// Reverse order
		const newOrder = [initialProducts[1], initialProducts[0]];

		await act(async () => {
			result.current.mutate(newOrder);
		});

		// Check cache was updated optimistically
		await waitFor(() => {
			const cached = queryClient.getQueryData<ProductResponse[]>(
				QUERY_KEYS.PRODUCTS,
			);
			expect(cached?.[0].id).toBe("prod-b");
			expect(cached?.[0].sort_order).toBe(0);
			expect(cached?.[1].id).toBe("prod-a");
			expect(cached?.[1].sort_order).toBe(1);
		});
	});
});
