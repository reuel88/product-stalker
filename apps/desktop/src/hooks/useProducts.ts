import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";
import type { ProductResponse } from "@/types/product";

export interface CreateProductInput {
	name: string;
	url: string;
	description?: string | null;
	notes?: string | null;
}

export interface UpdateProductInput {
	name?: string | null;
	url?: string | null;
	description?: string | null;
	notes?: string | null;
}

export function useProducts() {
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
