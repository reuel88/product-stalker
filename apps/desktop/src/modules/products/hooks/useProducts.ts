import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";
import type { ProductResponse } from "@/modules/products/types";

export interface CreateProductInput {
	name: string;
	description?: string | null;
	notes?: string | null;
}

export interface UpdateProductInput {
	name?: string | null;
	description?: string | null;
	notes?: string | null;
}

/** Return type for the useProducts hook */
export interface UseProductsReturn {
	/** Array of all products (undefined while loading) */
	products: ProductResponse[] | undefined;
	/** Whether the initial fetch is in progress */
	isLoading: boolean;
	/** Error from the last query, if any */
	error: Error | null;
	/** Async function to create a new product */
	createProduct: (input: CreateProductInput) => Promise<ProductResponse>;
	/** Whether a create operation is in progress */
	isCreating: boolean;
	/** Async function to update a product by id */
	updateProduct: (args: {
		id: string;
		input: UpdateProductInput;
	}) => Promise<ProductResponse>;
	/** Whether an update operation is in progress */
	isUpdating: boolean;
	/** Async function to delete a product by id */
	deleteProduct: (id: string) => Promise<void>;
	/** Whether a delete operation is in progress */
	isDeleting: boolean;
}

/**
 * Hook for managing products with CRUD operations.
 *
 * Provides reactive access to the products list and mutation functions
 * for creating, updating, and deleting products. All mutations automatically
 * invalidate the products query to keep the UI in sync.
 *
 * @returns Object containing:
 *   - products: Array of all products (undefined while loading)
 *   - isLoading: Whether the initial fetch is in progress
 *   - error: Error from the last query, if any
 *   - createProduct: Async function to create a new product
 *   - isCreating: Whether a create operation is in progress
 *   - updateProduct: Async function to update a product by id
 *   - isUpdating: Whether an update operation is in progress
 *   - deleteProduct: Async function to delete a product by id
 *   - isDeleting: Whether a delete operation is in progress
 */
export function useProducts(): UseProductsReturn {
	const queryClient = useQueryClient();

	const {
		data: products,
		isLoading,
		error,
	} = useQuery({
		queryKey: QUERY_KEYS.PRODUCTS,
		queryFn: () => invoke<ProductResponse[]>(COMMANDS.GET_PRODUCTS),
	});

	const createMutation = useMutation({
		mutationFn: (input: CreateProductInput) =>
			invoke<ProductResponse>(COMMANDS.CREATE_PRODUCT, { input }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: QUERY_KEYS.PRODUCTS });
		},
	});

	const updateMutation = useMutation({
		mutationFn: ({ id, input }: { id: string; input: UpdateProductInput }) =>
			invoke<ProductResponse>(COMMANDS.UPDATE_PRODUCT, { id, input }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: QUERY_KEYS.PRODUCTS });
		},
	});

	const deleteMutation = useMutation({
		mutationFn: (id: string) => invoke<void>(COMMANDS.DELETE_PRODUCT, { id }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: QUERY_KEYS.PRODUCTS });
		},
	});

	return {
		products,
		isLoading,
		error,
		createProduct: createMutation.mutateAsync,
		isCreating: createMutation.isPending,
		updateProduct: updateMutation.mutateAsync,
		isUpdating: updateMutation.isPending,
		deleteProduct: deleteMutation.mutateAsync,
		isDeleting: deleteMutation.isPending,
	};
}
