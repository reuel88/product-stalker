import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDialogDrag } from "@/components/ui/use-dialog-drag";

describe("useDialogDrag", () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it("should have initial state with zero offset and not dragging", () => {
		const { result } = renderHook(() => useDialogDrag());

		expect(result.current.offset).toEqual({ x: 0, y: 0 });
		expect(result.current.isDragging).toBe(false);
	});

	it("should update offset when dragging", () => {
		const { result } = renderHook(() => useDialogDrag());

		// Simulate pointerdown on a non-interactive element
		act(() => {
			const event = {
				target: document.createElement("div"),
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(true);

		// Simulate pointermove
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 150, clientY: 250 }),
			);
		});

		expect(result.current.offset).toEqual({ x: 50, y: 50 });
	});

	it("should stop dragging on pointerup", () => {
		const { result } = renderHook(() => useDialogDrag());

		act(() => {
			const event = {
				target: document.createElement("div"),
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(true);

		act(() => {
			document.dispatchEvent(new MouseEvent("pointerup"));
		});

		expect(result.current.isDragging).toBe(false);
	});

	it("should not start dragging when clicking on a button", () => {
		const { result } = renderHook(() => useDialogDrag());

		const button = document.createElement("button");
		const parent = document.createElement("div");
		parent.appendChild(button);

		act(() => {
			const event = {
				target: button,
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(false);
	});

	it("should not start dragging when clicking on an input", () => {
		const { result } = renderHook(() => useDialogDrag());

		const input = document.createElement("input");

		act(() => {
			const event = {
				target: input,
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(false);
	});

	it("should not start dragging when clicking on an element inside a button", () => {
		const { result } = renderHook(() => useDialogDrag());

		const button = document.createElement("button");
		const span = document.createElement("span");
		button.appendChild(span);

		act(() => {
			const event = {
				target: span,
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(false);
	});

	it("should accumulate offset across multiple drag operations", () => {
		const { result } = renderHook(() => useDialogDrag());

		// First drag: move 50px right
		act(() => {
			const event = {
				target: document.createElement("div"),
				clientX: 100,
				clientY: 100,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 150, clientY: 100 }),
			);
		});

		act(() => {
			document.dispatchEvent(new MouseEvent("pointerup"));
		});

		expect(result.current.offset).toEqual({ x: 50, y: 0 });

		// Second drag: move 30px down
		act(() => {
			const event = {
				target: document.createElement("div"),
				clientX: 200,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 200, clientY: 230 }),
			);
		});

		act(() => {
			document.dispatchEvent(new MouseEvent("pointerup"));
		});

		expect(result.current.offset).toEqual({ x: 50, y: 30 });
	});

	it("should reset offset and dragging state", () => {
		const { result } = renderHook(() => useDialogDrag());

		// Drag to create offset
		act(() => {
			const event = {
				target: document.createElement("div"),
				clientX: 100,
				clientY: 100,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 200, clientY: 300 }),
			);
		});

		act(() => {
			result.current.reset();
		});

		expect(result.current.offset).toEqual({ x: 0, y: 0 });
		expect(result.current.isDragging).toBe(false);
	});

	it("should clean up event listeners on unmount", () => {
		const removeSpy = vi.spyOn(document, "removeEventListener");
		const { result, unmount } = renderHook(() => useDialogDrag());

		// Start dragging to attach listeners
		act(() => {
			const event = {
				target: document.createElement("div"),
				clientX: 100,
				clientY: 100,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		unmount();

		expect(removeSpy).toHaveBeenCalledWith("pointermove", expect.any(Function));
		expect(removeSpy).toHaveBeenCalledWith("pointerup", expect.any(Function));

		removeSpy.mockRestore();
	});
});
