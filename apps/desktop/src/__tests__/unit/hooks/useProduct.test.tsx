import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants";
import { useProduct } from "@/modules/products/hooks/useProduct";
import { createMockProduct } from "../../mocks/data";
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

describe("useProduct", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	describe("fetching product", () => {
		it("should fetch product by id", async () => {
			const mockProduct = createMockProduct({
				id: "product-123",
				name: "Test Product",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: mockProduct,
			});

			const { result } = renderHook(() => useProduct("product-123"), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoading).toBe(true);

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(result.current.product).toEqual(mockProduct);
			expect(result.current.error).toBeNull();
		});

		it("should return loading state while fetching", async () => {
			const mockProduct = createMockProduct();
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: mockProduct,
			});

			const { result } = renderHook(() => useProduct("product-1"), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoading).toBe(true);
			expect(result.current.product).toBeUndefined();

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});
		});

		it("should not fetch when productId is empty", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: null,
			});

			const { result } = renderHook(() => useProduct(""), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(getMockedInvoke()).not.toHaveBeenCalledWith(
				COMMANDS.GET_PRODUCT,
				expect.anything(),
			);
		});

		it("should call invoke with correct parameters", async () => {
			const mockProduct = createMockProduct({ id: "test-id" });
			mockInvokeMultiple({
				[COMMANDS.GET_PRODUCT]: mockProduct,
			});

			const { result } = renderHook(() => useProduct("test-id"), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(getMockedInvoke()).toHaveBeenCalledWith(COMMANDS.GET_PRODUCT, {
				id: "test-id",
			});
		});
	});

	describe("error handling", () => {
		it("should return error when fetch fails", async () => {
			mockInvokeError(COMMANDS.GET_PRODUCT, "Product not found");

			const { result } = renderHook(() => useProduct("nonexistent-id"), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(result.current.error).toBeInstanceOf(Error);
			expect(result.current.error?.message).toBe("Product not found");
			expect(result.current.product).toBeUndefined();
		});
	});
});
