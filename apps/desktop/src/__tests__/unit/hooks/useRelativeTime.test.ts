import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useRelativeTime } from "@/hooks/useRelativeTime";

describe("useRelativeTime", () => {
	beforeEach(() => {
		vi.useFakeTimers();
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it("should return empty string for null input", () => {
		const { result } = renderHook(() => useRelativeTime(null));
		expect(result.current).toBe("");
	});

	it("should return 'Just now' for less than 1 minute ago", () => {
		const now = new Date();
		const thirtySecondsAgo = new Date(now.getTime() - 30_000).toISOString();

		vi.setSystemTime(now);

		const { result } = renderHook(() => useRelativeTime(thirtySecondsAgo));
		expect(result.current).toBe("Just now");
	});

	it("should return minutes format for 1-59 minutes ago", () => {
		const now = new Date();
		const fiveMinutesAgo = new Date(now.getTime() - 5 * 60_000).toISOString();

		vi.setSystemTime(now);

		const { result } = renderHook(() => useRelativeTime(fiveMinutesAgo));
		expect(result.current).toBe("5m ago");
	});

	it("should return hours format for 1-23 hours ago", () => {
		const now = new Date();
		const threeHoursAgo = new Date(
			now.getTime() - 3 * 60 * 60_000,
		).toISOString();

		vi.setSystemTime(now);

		const { result } = renderHook(() => useRelativeTime(threeHoursAgo));
		expect(result.current).toBe("3h ago");
	});

	it("should return days format for 24+ hours ago", () => {
		const now = new Date();
		const twoDaysAgo = new Date(
			now.getTime() - 2 * 24 * 60 * 60_000,
		).toISOString();

		vi.setSystemTime(now);

		const { result } = renderHook(() => useRelativeTime(twoDaysAgo));
		expect(result.current).toBe("2d ago");
	});

	it("should return 'Unknown' for invalid date strings", () => {
		const { result } = renderHook(() => useRelativeTime("invalid-date"));
		expect(result.current).toBe("Unknown");
	});

	it("should update time on interval", () => {
		const now = new Date();
		const thirtySecondsAgo = new Date(now.getTime() - 30_000).toISOString();

		vi.setSystemTime(now);

		const { result } = renderHook(() => useRelativeTime(thirtySecondsAgo));
		expect(result.current).toBe("Just now");

		// Advance time by 1 minute (60 seconds total)
		act(() => {
			vi.advanceTimersByTime(60_000);
		});

		expect(result.current).toBe("1m ago");
	});

	it("should update when dateStr changes", () => {
		const now = new Date();
		vi.setSystemTime(now);

		const fiveMinutesAgo = new Date(now.getTime() - 5 * 60_000).toISOString();
		const tenMinutesAgo = new Date(now.getTime() - 10 * 60_000).toISOString();

		const { result, rerender } = renderHook(
			({ dateStr }) => useRelativeTime(dateStr),
			{ initialProps: { dateStr: fiveMinutesAgo } },
		);

		expect(result.current).toBe("5m ago");

		rerender({ dateStr: tenMinutesAgo });
		expect(result.current).toBe("10m ago");
	});

	it("should clear interval when unmounted", () => {
		const clearIntervalSpy = vi.spyOn(global, "clearInterval");
		const now = new Date();
		vi.setSystemTime(now);

		const oneMinuteAgo = new Date(now.getTime() - 60_000).toISOString();

		const { unmount } = renderHook(() => useRelativeTime(oneMinuteAgo));
		unmount();

		expect(clearIntervalSpy).toHaveBeenCalled();
		clearIntervalSpy.mockRestore();
	});

	it("should handle empty string dateStr", () => {
		const { result, rerender } = renderHook(
			({ dateStr }) => useRelativeTime(dateStr),
			{ initialProps: { dateStr: null as string | null } },
		);

		expect(result.current).toBe("");

		// Change to a valid date
		const now = new Date();
		vi.setSystemTime(now);
		const oneMinuteAgo = new Date(now.getTime() - 60_000).toISOString();

		rerender({ dateStr: oneMinuteAgo });
		expect(result.current).toBe("1m ago");

		// Change back to null
		rerender({ dateStr: null });
		expect(result.current).toBe("");
	});
});
