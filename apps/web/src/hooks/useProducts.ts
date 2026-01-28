import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

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
		queryKey: ["products"],
		queryFn: () => invoke<ProductResponse[]>("get_products"),
	});

	const createMutation = useMutation({
		mutationFn: (input: CreateProductInput) =>
			invoke<ProductResponse>("create_product", { input }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["products"] });
		},
	});

	const updateMutation = useMutation({
		mutationFn: ({ id, input }: { id: string; input: UpdateProductInput }) =>
			invoke<ProductResponse>("update_product", { id, input }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["products"] });
		},
	});

	const deleteMutation = useMutation({
		mutationFn: (id: string) => invoke<void>("delete_product", { id }),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ["products"] });
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
