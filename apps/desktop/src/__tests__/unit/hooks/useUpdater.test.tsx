import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants";
import { useUpdater } from "@/modules/settings/hooks/useUpdater";
import {
	createMockUpdateAvailable,
	createMockUpdateInfo,
} from "../../mocks/data";
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

describe("useUpdater", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
	});

	describe("fetching current version", () => {
		it("should fetch current version successfully", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoadingVersion).toBe(true);

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			expect(result.current.currentVersion).toBe("1.0.0");
		});

		it("should handle loading state", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			expect(result.current.isLoadingVersion).toBe(true);
			expect(result.current.currentVersion).toBeUndefined();

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});
		});
	});

	describe("checkForUpdate mutation", () => {
		it("should check for updates and find none available", async () => {
			const noUpdate = createMockUpdateInfo();
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: noUpdate,
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			const updateInfo = await result.current.checkForUpdateAsync();

			expect(updateInfo.available).toBe(false);
			expect(updateInfo.version).toBeNull();
		});

		it("should check for updates and find one available", async () => {
			const updateAvailable = createMockUpdateAvailable(
				"1.1.0",
				"New features",
			);
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: updateAvailable,
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			const updateInfo = await result.current.checkForUpdateAsync();

			expect(updateInfo.available).toBe(true);
			expect(updateInfo.version).toBe("1.1.0");
			expect(updateInfo.body).toBe("New features");
		});

		it("should return isChecking state", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: createMockUpdateInfo(),
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			// isChecking should be a boolean
			expect(typeof result.current.isChecking).toBe("boolean");

			await result.current.checkForUpdateAsync();

			// After completion, isChecking should be false
			await waitFor(() => {
				expect(result.current.isChecking).toBe(false);
			});
		});

		it("should handle check for update error", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_CURRENT_VERSION) {
					return Promise.resolve("1.0.0");
				}
				if (cmd === COMMANDS.CHECK_FOR_UPDATE) {
					return Promise.reject(new Error("Network error"));
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			await expect(result.current.checkForUpdateAsync()).rejects.toThrow(
				"Network error",
			);

			await waitFor(() => {
				expect(result.current.checkError).toBeTruthy();
			});
		});

		it("should update updateInfo after check", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: updateAvailable,
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			expect(result.current.updateInfo).toBeUndefined();

			await result.current.checkForUpdateAsync();

			// After check completes, updateInfo should be updated
			await waitFor(() => {
				expect(result.current.updateInfo?.available).toBe(true);
			});
			expect(result.current.updateInfo?.version).toBe("2.0.0");
		});
	});

	describe("installUpdate mutation", () => {
		it("should install update successfully", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE]: undefined,
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			await result.current.installUpdateAsync();

			expect(getMockedInvoke()).toHaveBeenCalledWith(
				COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE,
			);
		});

		it("should return isInstalling state", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE]: undefined,
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			// isInstalling should be a boolean
			expect(typeof result.current.isInstalling).toBe("boolean");

			await result.current.installUpdateAsync();

			// After completion, isInstalling should be false
			await waitFor(() => {
				expect(result.current.isInstalling).toBe(false);
			});
		});

		it("should handle install error", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_CURRENT_VERSION) {
					return Promise.resolve("1.0.0");
				}
				if (cmd === COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE) {
					return Promise.reject(new Error("Download failed"));
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			await expect(result.current.installUpdateAsync()).rejects.toThrow(
				"Download failed",
			);

			await waitFor(() => {
				expect(result.current.installError).toBeTruthy();
			});
		});
	});

	describe("fire and forget methods", () => {
		it("should support checkForUpdate (non-async)", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: createMockUpdateAvailable(),
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			result.current.checkForUpdate();

			await waitFor(() => {
				expect(result.current.updateInfo?.available).toBe(true);
			});
		});

		it("should support installUpdate (non-async)", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE]: undefined,
			});

			const { result } = renderHook(() => useUpdater(), {
				wrapper: createWrapper(),
			});

			await waitFor(() => {
				expect(result.current.isLoadingVersion).toBe(false);
			});

			result.current.installUpdate();

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE,
				);
			});
		});
	});
});
