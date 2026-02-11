import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants";
import {
	useAvailability,
	useAvailabilityHistory,
	useCheckAllAvailability,
} from "@/modules/products/hooks/useAvailability";
import {
	createMockAvailabilityCheck,
	createMockBulkCheckSummary,
} from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { createHookWrapper } from "../../test-utils";

describe("useAvailability", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	describe("fetching latest availability", () => {
		it("should fetch latest availability check for product", async () => {
			const mockCheck = createMockAvailabilityCheck({
				product_id: "product-1",
				status: "in_stock",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_LATEST_AVAILABILITY]: mockCheck,
			});

			const { result } = renderHook(() => useAvailability("product-1"), {
				wrapper: createHookWrapper(),
			});

			expect(result.current.isLoading).toBe(true);

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(result.current.latestCheck).toEqual(mockCheck);
			expect(result.current.error).toBeNull();
		});

		it("should return loading state while fetching", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});

			const { result } = renderHook(() => useAvailability("product-1"), {
				wrapper: createHookWrapper(),
			});

			expect(result.current.isLoading).toBe(true);
			expect(result.current.latestCheck).toBeUndefined();

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});
		});

		it("should not fetch when productId is empty", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
			});

			const { result } = renderHook(() => useAvailability(""), {
				wrapper: createHookWrapper(),
			});

			// Should not be loading since query is disabled
			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(getMockedInvoke()).not.toHaveBeenCalledWith(
				COMMANDS.GET_LATEST_AVAILABILITY,
				expect.anything(),
			);
		});
	});

	describe("checkAvailability mutation", () => {
		it("should invoke check availability command", async () => {
			const mockCheck = createMockAvailabilityCheck({
				product_id: "product-1",
				status: "out_of_stock",
			});
			mockInvokeMultiple({
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.CHECK_AVAILABILITY]: mockCheck,
			});

			const { result } = renderHook(() => useAvailability("product-1"), {
				wrapper: createHookWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			await act(async () => {
				await result.current.checkAvailability();
			});

			expect(getMockedInvoke()).toHaveBeenCalledWith(
				COMMANDS.CHECK_AVAILABILITY,
				{ productId: "product-1" },
			);
		});

		it("should return isChecking state during mutation", async () => {
			const mockCheck = createMockAvailabilityCheck();
			mockInvokeMultiple({
				[COMMANDS.GET_LATEST_AVAILABILITY]: null,
				[COMMANDS.CHECK_AVAILABILITY]: mockCheck,
			});

			const { result } = renderHook(() => useAvailability("product-1"), {
				wrapper: createHookWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(result.current.isChecking).toBe(false);

			await act(async () => {
				await result.current.checkAvailability();
			});

			await waitFor(() => {
				expect(result.current.isChecking).toBe(false);
			});
		});
	});
});

describe("useAvailabilityHistory", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	it("should fetch availability history for product", async () => {
		const mockHistory = [
			createMockAvailabilityCheck({ status: "in_stock" }),
			createMockAvailabilityCheck({ status: "out_of_stock" }),
		];
		mockInvokeMultiple({
			[COMMANDS.GET_AVAILABILITY_HISTORY]: mockHistory,
		});

		const { result } = renderHook(() => useAvailabilityHistory("product-1"), {
			wrapper: createHookWrapper(),
		});

		await waitFor(() => {
			expect(result.current.isLoading).toBe(false);
		});

		expect(result.current.history).toEqual(mockHistory);
		expect(result.current.error).toBeNull();
	});

	it("should fetch history with limit parameter", async () => {
		const mockHistory = [createMockAvailabilityCheck()];
		mockInvokeMultiple({
			[COMMANDS.GET_AVAILABILITY_HISTORY]: mockHistory,
		});

		const { result } = renderHook(
			() => useAvailabilityHistory("product-1", 5),
			{
				wrapper: createHookWrapper(),
			},
		);

		await waitFor(() => {
			expect(result.current.isLoading).toBe(false);
		});

		expect(getMockedInvoke()).toHaveBeenCalledWith(
			COMMANDS.GET_AVAILABILITY_HISTORY,
			{ productId: "product-1", limit: 5 },
		);
	});

	it("should return loading state while fetching", async () => {
		mockInvokeMultiple({
			[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
		});

		const { result } = renderHook(() => useAvailabilityHistory("product-1"), {
			wrapper: createHookWrapper(),
		});

		expect(result.current.isLoading).toBe(true);

		await waitFor(() => {
			expect(result.current.isLoading).toBe(false);
		});
	});

	it("should not fetch when productId is empty", async () => {
		mockInvokeMultiple({
			[COMMANDS.GET_AVAILABILITY_HISTORY]: [],
		});

		const { result } = renderHook(() => useAvailabilityHistory(""), {
			wrapper: createHookWrapper(),
		});

		await waitFor(() => {
			expect(result.current.isLoading).toBe(false);
		});

		expect(getMockedInvoke()).not.toHaveBeenCalledWith(
			COMMANDS.GET_AVAILABILITY_HISTORY,
			expect.anything(),
		);
	});
});

describe("useCheckAllAvailability", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	it("should invoke bulk check availability command", async () => {
		const mockSummary = createMockBulkCheckSummary({
			total: 5,
			successful: 4,
			failed: 1,
		});
		mockInvokeMultiple({
			[COMMANDS.CHECK_ALL_AVAILABILITY]: mockSummary,
		});

		const { result } = renderHook(() => useCheckAllAvailability(), {
			wrapper: createHookWrapper(),
		});

		await act(async () => {
			await result.current.checkAllAvailability();
		});

		expect(getMockedInvoke()).toHaveBeenCalledWith(
			COMMANDS.CHECK_ALL_AVAILABILITY,
		);
	});

	it("should return isCheckingAll state during mutation", async () => {
		const mockSummary = createMockBulkCheckSummary();
		mockInvokeMultiple({
			[COMMANDS.CHECK_ALL_AVAILABILITY]: mockSummary,
		});

		const { result } = renderHook(() => useCheckAllAvailability(), {
			wrapper: createHookWrapper(),
		});

		expect(result.current.isCheckingAll).toBe(false);

		await act(async () => {
			await result.current.checkAllAvailability();
		});

		await waitFor(() => {
			expect(result.current.isCheckingAll).toBe(false);
		});
	});

	it("should return lastSummary after successful check", async () => {
		const mockSummary = createMockBulkCheckSummary({
			total: 10,
			successful: 8,
			failed: 2,
			back_in_stock_count: 3,
		});
		mockInvokeMultiple({
			[COMMANDS.CHECK_ALL_AVAILABILITY]: mockSummary,
		});

		const { result } = renderHook(() => useCheckAllAvailability(), {
			wrapper: createHookWrapper(),
		});

		await act(async () => {
			await result.current.checkAllAvailability();
		});

		await waitFor(() => {
			expect(result.current.lastSummary).toEqual(mockSummary);
		});
	});
});
