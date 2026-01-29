import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants";
import { useProducts } from "@/hooks/useProducts";
import { createMockProduct, createMockProducts } from "../../mocks/data";
import {
	getMockedInvoke,
	mockInvokeError,
	mockInvokeMultiple,
} from "../../mocks/tauri";

function createWrapper() {
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: { retry: false, gcTime: 0 },
			mutations: { retry: false },
		},
	});
	return function Wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		);
	};
}

describe("useProducts", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	describe("fetching products", () => {
		it("should fetch products successfully", async () => {
			const mockProducts = createMockProducts(3);
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: mockProducts,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoading).toBe(true);

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(result.current.products).toEqual(mockProducts);
			expect(result.current.error).toBeNull();
		});

		it("should handle loading state", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoading).toBe(true);
			expect(result.current.products).toBeUndefined();

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});
		});

		it("should handle fetch error", async () => {
			mockInvokeError(COMMANDS.GET_PRODUCTS, "Failed to fetch products");

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.error).toBeTruthy();
			});

			expect(result.current.products).toBeUndefined();
		});
	});

	describe("createProduct mutation", () => {
		it("should create a product successfully", async () => {
			const newProduct = createMockProduct({ name: "New Product" });
			const existingProducts = createMockProducts(2);

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: existingProducts,
				[COMMANDS.CREATE_PRODUCT]: newProduct,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			const createdProduct = await result.current.createProduct({
				name: "New Product",
				url: "https://example.com/new",
			});

			expect(createdProduct).toEqual(newProduct);
		});

		it("should return isCreating state", async () => {
			const newProduct = createMockProduct();
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
				[COMMANDS.CREATE_PRODUCT]: newProduct,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			// isCreating should be a boolean
			expect(typeof result.current.isCreating).toBe("boolean");

			await result.current.createProduct({
				name: "Test",
				url: "https://test.com",
			});

			// After completion, isCreating should be false
			await waitFor(() => {
				expect(result.current.isCreating).toBe(false);
			});
		});
	});

	describe("updateProduct mutation", () => {
		it("should update a product successfully", async () => {
			const existingProduct = createMockProduct({ id: "1", name: "Old Name" });
			const updatedProduct = { ...existingProduct, name: "New Name" };

			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [existingProduct],
				[COMMANDS.UPDATE_PRODUCT]: updatedProduct,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			const updated = await result.current.updateProduct({
				id: "1",
				input: { name: "New Name" },
			});

			expect(updated.name).toBe("New Name");
		});

		it("should return isUpdating state", async () => {
			const product = createMockProduct();
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.UPDATE_PRODUCT]: product,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			// isUpdating should be a boolean
			expect(typeof result.current.isUpdating).toBe("boolean");

			await result.current.updateProduct({
				id: product.id,
				input: { name: "Updated" },
			});

			// After completion, isUpdating should be false
			await waitFor(() => {
				expect(result.current.isUpdating).toBe(false);
			});
		});
	});

	describe("deleteProduct mutation", () => {
		it("should delete a product successfully", async () => {
			const product = createMockProduct({ id: "1" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.DELETE_PRODUCT]: undefined,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			await result.current.deleteProduct("1");

			expect(getMockedInvoke()).toHaveBeenCalledWith(COMMANDS.DELETE_PRODUCT, {
				id: "1",
			});
		});

		it("should return isDeleting state", async () => {
			const product = createMockProduct();
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [product],
				[COMMANDS.DELETE_PRODUCT]: undefined,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			// isDeleting should be a boolean
			expect(typeof result.current.isDeleting).toBe("boolean");

			await result.current.deleteProduct(product.id);

			// After completion, isDeleting should be false
			await waitFor(() => {
				expect(result.current.isDeleting).toBe(false);
			});
		});
	});

	describe("cache invalidation", () => {
		it("should invalidate products cache after create", async () => {
			const newProduct = createMockProduct();
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCTS]: [],
				[COMMANDS.CREATE_PRODUCT]: newProduct,
			});

			const { result } = renderHook(() => useProducts(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			await result.current.createProduct({
				name: "Test",
				url: "https://test.com",
			});

			// The query should have been called twice: initial + after invalidation
			await waitFor(() => {
				const calls = getMockedInvoke().mock.calls.filter(
					(call) => call[0] === COMMANDS.GET_PRODUCTS,
				);
				expect(calls.length).toBeGreaterThanOrEqual(2);
			});
		});
	});
});
