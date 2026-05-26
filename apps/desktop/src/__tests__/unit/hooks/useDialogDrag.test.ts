import { act, renderHook } from "@testing-library/react";
import type { RefObject } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDialogDrag } from "@/components/ui/use-dialog-drag";

function createMockPopupRef(
	rect: Partial<DOMRect> = {},
): RefObject<HTMLDivElement | null> {
	const el = document.createElement("div");
	vi.spyOn(el, "getBoundingClientRect").mockReturnValue({
		width: 400,
		height: 300,
		top: 100,
		left: 200,
		right: 600,
		bottom: 400,
		x: 200,
		y: 100,
		toJSON: vi.fn(),
		...rect,
	});
	return { current: el };
}

describe("useDialogDrag", () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it("should have initial state with zero offset and not dragging", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		expect(result.current.offset).toEqual({ x: 0, y: 0 });
		expect(result.current.isDragging).toBe(false);
	});

	it("should update offset when dragging", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		// Simulate pointerdown on a non-interactive element
		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
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
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
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
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

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
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

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
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

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
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		// First drag: move 50px right
		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
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
				button: 0,
				isPrimary: true,
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
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		// Drag to create offset
		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
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
		const popupRef = createMockPopupRef();
		const { result, unmount } = renderHook(() => useDialogDrag(popupRef));

		// Start dragging to attach listeners
		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
				clientX: 100,
				clientY: 100,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		unmount();

		expect(removeSpy).toHaveBeenCalledWith("pointermove", expect.any(Function));
		expect(removeSpy).toHaveBeenCalledWith("pointerup", expect.any(Function));
		expect(removeSpy).toHaveBeenCalledWith(
			"pointercancel",
			expect.any(Function),
		);

		removeSpy.mockRestore();
	});

	it("should not start dragging on right-click (non-primary button)", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 2,
				isPrimary: true,
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(false);
	});

	it("should not start dragging for a non-primary pointer", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: false,
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(false);
	});

	it("should stop dragging on pointercancel", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogDrag(popupRef));

		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
				clientX: 100,
				clientY: 200,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		expect(result.current.isDragging).toBe(true);

		act(() => {
			document.dispatchEvent(new MouseEvent("pointercancel"));
		});

		expect(result.current.isDragging).toBe(false);
	});

	it("should clamp drag to prevent dialog from going off-screen right/bottom", () => {
		// Dialog at left:200, top:100, width:400, height:300 → right:600, bottom:400
		// Viewport: 1024x768
		vi.spyOn(window, "innerWidth", "get").mockReturnValue(1024);
		vi.spyOn(window, "innerHeight", "get").mockReturnValue(768);

		const popupRef = createMockPopupRef({
			left: 200,
			top: 100,
			right: 600,
			bottom: 400,
			width: 400,
			height: 300,
		});
		const { result } = renderHook(() => useDialogDrag(popupRef));

		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
				clientX: 400,
				clientY: 250,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		// Try to drag 500px right — would put right edge at 1100, past viewport (1024)
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 900, clientY: 250 }),
			);
		});

		// Should be clamped: max dx = 1024 - 600 = 424
		expect(result.current.offset.x).toBe(424);
		expect(result.current.offset.y).toBe(0);

		act(() => {
			document.dispatchEvent(new MouseEvent("pointerup"));
		});

		// Now drag down past bottom edge
		// After first drag, dialog conceptually moved — recapture rect for next drag
		const popupRef2 = createMockPopupRef({
			left: 624,
			top: 100,
			right: 1024,
			bottom: 400,
			width: 400,
			height: 300,
		});
		const { result: result2 } = renderHook(() => useDialogDrag(popupRef2));

		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
				clientX: 800,
				clientY: 250,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result2.current.handlePointerDown(event);
		});

		// Try to drag 500px down — would put bottom at 900, past viewport (768)
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 800, clientY: 750 }),
			);
		});

		// Should be clamped: max dy = 768 - 400 = 368
		expect(result2.current.offset.y).toBe(368);
	});

	it("should clamp drag to prevent dialog from going off-screen left/top", () => {
		vi.spyOn(window, "innerWidth", "get").mockReturnValue(1024);
		vi.spyOn(window, "innerHeight", "get").mockReturnValue(768);

		// Dialog near top-left: left:50, top:30
		const popupRef = createMockPopupRef({
			left: 50,
			top: 30,
			right: 450,
			bottom: 330,
			width: 400,
			height: 300,
		});
		const { result } = renderHook(() => useDialogDrag(popupRef));

		act(() => {
			const event = {
				target: document.createElement("div"),
				button: 0,
				isPrimary: true,
				clientX: 200,
				clientY: 150,
				preventDefault: vi.fn(),
			} as unknown as React.PointerEvent;
			result.current.handlePointerDown(event);
		});

		// Try to drag 100px left — would put left edge at -50
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 100, clientY: 50 }),
			);
		});

		// Should be clamped: min dx = -50 (so left stays at 0)
		expect(result.current.offset.x).toBe(-50);
		// dy = -100, would put top at -70 → clamped to top=0, so dy = -30
		expect(result.current.offset.y).toBe(-30);
	});
});
