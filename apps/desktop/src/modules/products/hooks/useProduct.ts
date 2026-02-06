import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";
import type { ProductResponse } from "@/modules/products/types";

/**
 * Hook for fetching a single product by ID.
 *
 * @param productId - The UUID of the product to fetch
 * @returns Object containing:
 *   - `product`: The product data (undefined while loading)
 *   - `isLoading`: Whether the query is in progress
 *   - `error`: Error from the query, if any
 */
export function useProduct(productId: string) {
	const {
		data: product,
		isLoading,
		error,
	} = useQuery({
		queryKey: QUERY_KEYS.product(productId),
		queryFn: () =>
			invoke<ProductResponse>(COMMANDS.GET_PRODUCT, { id: productId }),
		enabled: !!productId,
	});

	return {
		product,
		isLoading,
		error,
	};
}
