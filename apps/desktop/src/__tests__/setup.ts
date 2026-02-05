import "@testing-library/jest-dom";
import { beforeEach, vi } from "vitest";

// Mock ResizeObserver for Radix UI compatibility
class ResizeObserverMock {
	observe() {}
	unobserve() {}
	disconnect() {}
}

global.ResizeObserver = ResizeObserverMock;

// Mock pointer capture methods for Radix UI Select compatibility
if (typeof Element.prototype.hasPointerCapture === "undefined") {
	Element.prototype.hasPointerCapture = () => false;
}
if (typeof Element.prototype.setPointerCapture === "undefined") {
	Element.prototype.setPointerCapture = () => {};
}
if (typeof Element.prototype.releasePointerCapture === "undefined") {
	Element.prototype.releasePointerCapture = () => {};
}
if (typeof Element.prototype.scrollIntoView === "undefined") {
	Element.prototype.scrollIntoView = () => {};
}

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

// Mock @tauri-apps/api/event
vi.mock("@tauri-apps/api/event", () => ({
	listen: vi.fn(() => Promise.resolve(() => {})),
	emit: vi.fn(),
}));

// Mock @tauri-apps/plugin-opener
vi.mock("@tauri-apps/plugin-opener", () => ({
	openUrl: vi.fn(() => Promise.resolve()),
	openPath: vi.fn(() => Promise.resolve()),
	revealItemInDir: vi.fn(() => Promise.resolve()),
}));

// Clear all mocks between tests
beforeEach(() => {
	vi.clearAllMocks();
});
