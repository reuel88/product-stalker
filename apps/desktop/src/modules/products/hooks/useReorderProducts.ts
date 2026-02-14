import { useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";
import type { ProductResponse } from "@/modules/products/types";

interface ReorderUpdate {
	id: string;
	sort_order: number;
}

interface ReorderProductsInput {
	updates: ReorderUpdate[];
}

/**
 * Hook for reordering products via drag-and-drop.
 *
 * Provides an optimistic mutation that updates the cache immediately
 * and rolls back on error.
 */
export function useReorderProducts() {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (newOrder: ProductResponse[]) => {
			const input: ReorderProductsInput = {
				updates: newOrder.map((product, index) => ({
					id: product.id,
					sort_order: index,
				})),
			};
			return invoke<void>(COMMANDS.REORDER_PRODUCTS, { input });
		},
		onMutate: async (newOrder: ProductResponse[]) => {
			await queryClient.cancelQueries({ queryKey: QUERY_KEYS.PRODUCTS });

			const previous = queryClient.getQueryData<ProductResponse[]>(
				QUERY_KEYS.PRODUCTS,
			);

			queryClient.setQueryData<ProductResponse[]>(
				QUERY_KEYS.PRODUCTS,
				newOrder.map((product, index) => ({
					...product,
					sort_order: index,
				})),
			);

			return { previous };
		},
		onError: (_err, _newOrder, context) => {
			if (context?.previous) {
				queryClient.setQueryData(QUERY_KEYS.PRODUCTS, context.previous);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: QUERY_KEYS.PRODUCTS });
		},
	});
}
