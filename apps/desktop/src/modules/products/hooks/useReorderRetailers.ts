import { useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";
import type { ProductRetailerResponse } from "@/modules/products/types";

interface ReorderRetailerUpdate {
	id: string;
	sort_order: number;
}

interface ReorderRetailersInput {
	updates: ReorderRetailerUpdate[];
}

/**
 * Hook for reordering product retailers via drag-and-drop.
 *
 * Provides an optimistic mutation that updates the cache immediately
 * and rolls back on error.
 */
export function useReorderRetailers(productId: string) {
	const queryClient = useQueryClient();

	return useMutation({
		mutationFn: (newOrder: ProductRetailerResponse[]) => {
			const input: ReorderRetailersInput = {
				updates: newOrder.map((retailer, index) => ({
					id: retailer.id,
					sort_order: index,
				})),
			};
			return invoke<void>(COMMANDS.REORDER_PRODUCT_RETAILERS, { input });
		},
		onMutate: async (newOrder: ProductRetailerResponse[]) => {
			const queryKey = QUERY_KEYS.productRetailers(productId);
			await queryClient.cancelQueries({ queryKey });

			const previous =
				queryClient.getQueryData<ProductRetailerResponse[]>(queryKey);

			queryClient.setQueryData<ProductRetailerResponse[]>(
				queryKey,
				newOrder.map((retailer, index) => ({
					...retailer,
					sort_order: index,
				})),
			);

			return { previous };
		},
		onError: (_err, _newOrder, context) => {
			if (context?.previous) {
				queryClient.setQueryData(
					QUERY_KEYS.productRetailers(productId),
					context.previous,
				);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.productRetailers(productId),
			});
		},
	});
}
