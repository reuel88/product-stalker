import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";
import type { ProductRetailerResponse } from "@/modules/products/types";

export interface AddRetailerInput {
	product_id: string;
	url: string;
	label?: string | null;
}

/**
 * Hook for managing product-retailer links.
 *
 * Provides reactive access to retailer links for a product and mutations
 * for adding and removing retailers.
 *
 * @param productId - The UUID of the product
 */
export function useProductRetailers(productId: string) {
	const queryClient = useQueryClient();

	const {
		data: retailers,
		isLoading,
		error,
	} = useQuery({
		queryKey: QUERY_KEYS.productRetailers(productId),
		queryFn: () =>
			invoke<ProductRetailerResponse[]>(COMMANDS.GET_PRODUCT_RETAILERS, {
				productId,
			}),
		enabled: !!productId,
	});

	const addMutation = useMutation({
		mutationFn: (input: AddRetailerInput) =>
			invoke<ProductRetailerResponse>(COMMANDS.ADD_PRODUCT_RETAILER, {
				input,
			}),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.productRetailers(productId),
			});
		},
	});

	const removeMutation = useMutation({
		mutationFn: (id: string) =>
			invoke<void>(COMMANDS.REMOVE_PRODUCT_RETAILER, { id }),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.productRetailers(productId),
			});
		},
	});

	return {
		retailers,
		isLoading,
		error,
		addRetailer: addMutation.mutateAsync,
		isAdding: addMutation.isPending,
		removeRetailer: removeMutation.mutateAsync,
		isRemoving: removeMutation.isPending,
	};
}
