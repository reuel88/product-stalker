import { act, renderHook } from "@testing-library/react";
import type { RefObject } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDialogResize } from "@/components/ui/use-dialog-resize";

function createMockPopupRef(
	rect: Partial<DOMRect> = {},
): RefObject<HTMLDivElement | null> {
	const el = document.createElement("div");
	vi.spyOn(el, "getBoundingClientRect").mockReturnValue({
		width: 500,
		height: 400,
		top: 0,
		left: 0,
		right: 500,
		bottom: 400,
		x: 0,
		y: 0,
		toJSON: vi.fn(),
		...rect,
	});
	return { current: el };
}

describe("useDialogResize", () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it("should have initial state with null size and not resizing", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		expect(result.current.size).toEqual({ width: null, height: null });
		expect(result.current.isResizing).toBe(false);
		expect(result.current.resizeOffset).toEqual({ x: 0, y: 0 });
	});

	it("should expose all 8 resize directions", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		expect(result.current.directions).toEqual([
			"n",
			"s",
			"e",
			"w",
			"ne",
			"nw",
			"se",
			"sw",
		]);
	});

	it("should return handle props with correct cursor for each direction", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		const cursorMap: Record<string, string> = {
			n: "ns-resize",
			s: "ns-resize",
			e: "ew-resize",
			w: "ew-resize",
			ne: "nesw-resize",
			sw: "nesw-resize",
			nw: "nwse-resize",
			se: "nwse-resize",
		};

		for (const dir of result.current.directions) {
			const handleProps = result.current.getHandleProps(dir);
			expect(handleProps.style.cursor).toBe(cursorMap[dir]);
			expect(handleProps.style.position).toBe("absolute");
			expect(handleProps["data-resize-handle"]).toBe(dir);
			expect(handleProps.onPointerDown).toBeInstanceOf(Function);
		}
	});

	it("should capture initial popup size on first resize", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		// Start resizing from the east handle
		const handleProps = result.current.getHandleProps("e");
		act(() => {
			handleProps.onPointerDown({
				clientX: 500,
				clientY: 200,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		expect(result.current.isResizing).toBe(true);
		expect(result.current.size).toEqual({ width: 500, height: 400 });
	});

	it("should resize east direction correctly", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		// Start resize
		act(() => {
			result.current.getHandleProps("e").onPointerDown({
				clientX: 500,
				clientY: 200,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Move east by 100px
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 600, clientY: 200 }),
			);
		});

		expect(result.current.size.width).toBe(600);
		expect(result.current.size.height).toBe(400);
		expect(result.current.resizeOffset).toEqual({ x: 50, y: 0 });
	});

	it("should resize south direction correctly", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("s").onPointerDown({
				clientX: 250,
				clientY: 400,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 250, clientY: 500 }),
			);
		});

		expect(result.current.size.height).toBe(500);
		expect(result.current.size.width).toBe(500);
	});

	it("should resize west direction with offset adjustment", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("w").onPointerDown({
				clientX: 0,
				clientY: 200,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Move west handle left by 50px (increasing width)
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: -50, clientY: 200 }),
			);
		});

		expect(result.current.size.width).toBe(550);
		// West resize should produce negative x offset to anchor the right edge
		expect(result.current.resizeOffset.x).toBe(-25);
	});

	it("should resize north direction with offset adjustment", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("n").onPointerDown({
				clientX: 250,
				clientY: 0,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Move north handle up by 50px
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 250, clientY: -50 }),
			);
		});

		expect(result.current.size.height).toBe(450);
		expect(result.current.resizeOffset.y).toBe(-25);
	});

	it("should resize se corner (both width and height)", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("se").onPointerDown({
				clientX: 500,
				clientY: 400,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 600, clientY: 500 }),
			);
		});

		expect(result.current.size).toEqual({ width: 600, height: 500 });
		expect(result.current.resizeOffset).toEqual({ x: 50, y: 50 });
	});

	it("should enforce minimum width constraint", () => {
		const popupRef = createMockPopupRef({
			width: 400,
			height: 400,
			top: 0,
			left: 0,
			right: 400,
			bottom: 400,
		});
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("e").onPointerDown({
				clientX: 400,
				clientY: 200,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Try to shrink below 320px min width
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 200, clientY: 200 }),
			);
		});

		expect(result.current.size.width).toBe(320);
	});

	it("should enforce minimum height constraint", () => {
		const popupRef = createMockPopupRef({
			width: 500,
			height: 300,
			top: 0,
			left: 0,
			right: 500,
			bottom: 300,
		});
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("s").onPointerDown({
				clientX: 250,
				clientY: 300,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Try to shrink below 180px min height
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 250, clientY: 50 }),
			);
		});

		expect(result.current.size.height).toBe(180);
	});

	it("should stop resizing on pointerup", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("e").onPointerDown({
				clientX: 500,
				clientY: 200,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		expect(result.current.isResizing).toBe(true);

		act(() => {
			document.dispatchEvent(new MouseEvent("pointerup"));
		});

		expect(result.current.isResizing).toBe(false);
	});

	it("should reset size and offset", () => {
		const popupRef = createMockPopupRef();
		const { result } = renderHook(() => useDialogResize(popupRef));

		// Resize to create custom size
		act(() => {
			result.current.getHandleProps("se").onPointerDown({
				clientX: 500,
				clientY: 400,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 600, clientY: 500 }),
			);
		});

		act(() => {
			document.dispatchEvent(new MouseEvent("pointerup"));
		});

		expect(result.current.size).toEqual({ width: 600, height: 500 });

		act(() => {
			result.current.reset();
		});

		expect(result.current.size).toEqual({ width: null, height: null });
		expect(result.current.resizeOffset).toEqual({ x: 0, y: 0 });
		expect(result.current.isResizing).toBe(false);
	});

	it("should clean up event listeners on unmount", () => {
		const removeSpy = vi.spyOn(document, "removeEventListener");
		const popupRef = createMockPopupRef();
		const { result, unmount } = renderHook(() => useDialogResize(popupRef));

		// Start resizing to attach listeners
		act(() => {
			result.current.getHandleProps("e").onPointerDown({
				clientX: 500,
				clientY: 200,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		unmount();

		expect(removeSpy).toHaveBeenCalledWith("pointermove", expect.any(Function));
		expect(removeSpy).toHaveBeenCalledWith("pointerup", expect.any(Function));

		removeSpy.mockRestore();
	});

	it("should clamp east resize to viewport right edge", () => {
		vi.spyOn(window, "innerWidth", "get").mockReturnValue(800);
		vi.spyOn(window, "innerHeight", "get").mockReturnValue(600);

		// Dialog: left=100, right=600, width=500
		const popupRef = createMockPopupRef({
			left: 100,
			top: 50,
			right: 600,
			bottom: 450,
			width: 500,
			height: 400,
		});
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("e").onPointerDown({
				clientX: 600,
				clientY: 250,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Try to resize 500px east — max width = 800 - 100 = 700
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 1100, clientY: 250 }),
			);
		});

		expect(result.current.size.width).toBe(700);
	});

	it("should clamp south resize to viewport bottom edge", () => {
		vi.spyOn(window, "innerWidth", "get").mockReturnValue(800);
		vi.spyOn(window, "innerHeight", "get").mockReturnValue(600);

		// Dialog: top=200, bottom=500, height=300
		const popupRef = createMockPopupRef({
			left: 100,
			top: 200,
			right: 600,
			bottom: 500,
			width: 500,
			height: 300,
		});
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("s").onPointerDown({
				clientX: 350,
				clientY: 500,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Try to resize 400px south — max height = 600 - 200 = 400
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 350, clientY: 900 }),
			);
		});

		expect(result.current.size.height).toBe(400);
	});

	it("should clamp west resize to viewport left edge", () => {
		vi.spyOn(window, "innerWidth", "get").mockReturnValue(800);
		vi.spyOn(window, "innerHeight", "get").mockReturnValue(600);

		// Dialog: left=100, right=600, width=500
		const popupRef = createMockPopupRef({
			left: 100,
			top: 50,
			right: 600,
			bottom: 450,
			width: 500,
			height: 400,
		});
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("w").onPointerDown({
				clientX: 100,
				clientY: 250,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Try to resize 200px west — max width = right = 600
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: -100, clientY: 250 }),
			);
		});

		// maxW = 600 (right edge), prevW = 500, dx = -200, so prevW - dx = 700 → clamped to 600
		expect(result.current.size.width).toBe(600);
	});

	it("should clamp north resize to viewport top edge", () => {
		vi.spyOn(window, "innerWidth", "get").mockReturnValue(800);
		vi.spyOn(window, "innerHeight", "get").mockReturnValue(600);

		// Dialog: top=50, bottom=450, height=400
		const popupRef = createMockPopupRef({
			left: 100,
			top: 50,
			right: 600,
			bottom: 450,
			width: 500,
			height: 400,
		});
		const { result } = renderHook(() => useDialogResize(popupRef));

		act(() => {
			result.current.getHandleProps("n").onPointerDown({
				clientX: 350,
				clientY: 50,
				preventDefault: vi.fn(),
				stopPropagation: vi.fn(),
			} as unknown as React.PointerEvent);
		});

		// Try to resize 200px north — max height = bottom = 450
		act(() => {
			document.dispatchEvent(
				new MouseEvent("pointermove", { clientX: 350, clientY: -150 }),
			);
		});

		// maxH = 450 (bottom edge), prevH = 400, dy = -200, so prevH - dy = 600 → clamped to 450
		expect(result.current.size.height).toBe(450);
	});
});
