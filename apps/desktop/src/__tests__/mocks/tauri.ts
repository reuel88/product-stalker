import { invoke } from "@tauri-apps/api/core";
import { type Mock, vi } from "vitest";

type InvokeFunction = typeof invoke;

/**
 * Get the mocked invoke function
 */
export function getMockedInvoke(): Mock<InvokeFunction> {
	return invoke as Mock<InvokeFunction>;
}

/**
 * Mock a successful Tauri invoke call
 * @param command - The command to mock
 * @param response - The response to return
 */
export function mockInvoke<T>(command: string, response: T): void {
	const mockedInvoke = getMockedInvoke();
	mockedInvoke.mockImplementation((cmd: string) => {
		if (cmd === command) {
			return Promise.resolve(response);
		}
		return Promise.reject(new Error(`Unmocked command: ${cmd}`));
	});
}

/**
 * Mock multiple Tauri invoke calls
 * @param mocks - Record of command to response mappings
 */
export function mockInvokeMultiple(mocks: Record<string, unknown>): void {
	const mockedInvoke = getMockedInvoke();
	mockedInvoke.mockImplementation((cmd: string) => {
		if (cmd in mocks) {
			return Promise.resolve(mocks[cmd]);
		}
		return Promise.reject(new Error(`Unmocked command: ${cmd}`));
	});
}

/**
 * Mock a Tauri invoke call to throw an error
 * @param command - The command to mock
 * @param error - The error message or Error object
 */
export function mockInvokeError(command: string, error: string | Error): void {
	const mockedInvoke = getMockedInvoke();
	mockedInvoke.mockImplementation((cmd: string) => {
		if (cmd === command) {
			return Promise.reject(
				typeof error === "string" ? new Error(error) : error,
			);
		}
		return Promise.reject(new Error(`Unmocked command: ${cmd}`));
	});
}

/**
 * Mock a sequence of responses for the same command
 * Useful for testing retry logic or state changes
 * @param command - The command to mock
 * @param responses - Array of responses (can include Error objects for failures)
 */
export function mockInvokeSequence(
	command: string,
	responses: (unknown | Error)[],
): void {
	const mockedInvoke = getMockedInvoke();
	let callIndex = 0;
	mockedInvoke.mockImplementation((cmd: string) => {
		if (cmd === command) {
			const response = responses[callIndex];
			callIndex = Math.min(callIndex + 1, responses.length - 1);
			if (response instanceof Error) {
				return Promise.reject(response);
			}
			return Promise.resolve(response);
		}
		return Promise.reject(new Error(`Unmocked command: ${cmd}`));
	});
}

/**
 * Reset the invoke mock to its default state
 */
export function resetInvokeMock(): void {
	const mockedInvoke = getMockedInvoke();
	mockedInvoke.mockReset();
}
