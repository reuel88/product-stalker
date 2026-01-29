import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants";
import { useSettings } from "@/hooks/useSettings";
import { createMockSettings } from "../../mocks/data";
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

describe("useSettings", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	describe("fetching settings", () => {
		it("should fetch settings successfully", async () => {
			const mockSettings = createMockSettings();
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: mockSettings,
			});

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoading).toBe(true);

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			expect(result.current.settings).toEqual(mockSettings);
			expect(result.current.error).toBeNull();
		});

		it("should handle loading state", async () => {
			const mockSettings = createMockSettings();
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: mockSettings,
			});

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoading).toBe(true);
			expect(result.current.settings).toBeUndefined();

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});
		});

		it("should handle fetch error", async () => {
			mockInvokeError(COMMANDS.GET_SETTINGS, "Failed to fetch settings");

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.error).toBeTruthy();
			});

			expect(result.current.settings).toBeUndefined();
		});
	});

	describe("updateSettings mutation", () => {
		it("should update settings with updateSettings (fire and forget)", async () => {
			const currentSettings = createMockSettings();
			const updatedSettings = createMockSettings({ theme: "dark" });

			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: currentSettings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			result.current.updateSettings({ theme: "dark" });

			await waitFor(() => {
				expect(result.current.settings?.theme).toBe("dark");
			});
		});

		it("should update settings with updateSettingsAsync", async () => {
			const currentSettings = createMockSettings();
			const updatedSettings = createMockSettings({ enable_logging: false });

			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: currentSettings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			const updated = await result.current.updateSettingsAsync({
				enable_logging: false,
			});

			expect(updated.enable_logging).toBe(false);
		});

		it("should return isUpdating state", async () => {
			const settings = createMockSettings();
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.UPDATE_SETTINGS]: settings,
			});

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			// isUpdating should be a boolean
			expect(typeof result.current.isUpdating).toBe("boolean");

			await result.current.updateSettingsAsync({
				theme: "light",
			});

			// After completion, isUpdating should be false
			await waitFor(() => {
				expect(result.current.isUpdating).toBe(false);
			});
		});

		it("should handle update error", async () => {
			const settings = createMockSettings();
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS) {
					return Promise.resolve(settings);
				}
				if (cmd === COMMANDS.UPDATE_SETTINGS) {
					return Promise.reject(new Error("Failed to update"));
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoading).toBe(false);
			});

			await expect(
				result.current.updateSettingsAsync({ theme: "dark" }),
			).rejects.toThrow("Failed to update");
		});
	});

	describe("cache updates", () => {
		it("should update cache on mutation success", async () => {
			const currentSettings = createMockSettings({ theme: "light" });
			const updatedSettings = createMockSettings({ theme: "dark" });

			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: currentSettings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { result } = renderHook(() => useSettings(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.settings?.theme).toBe("light");
			});

			await result.current.updateSettingsAsync({ theme: "dark" });

			// After the mutation completes, the cache should be updated
			await waitFor(() => {
				expect(result.current.settings?.theme).toBe("dark");
			});
		});
	});
});
