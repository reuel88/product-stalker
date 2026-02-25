import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useDialogResize } from "@/components/ui/use-dialog-resize";

describe("useDialogResize", () => {
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it("should have initial state with null size and not resizing", () => {
		const { result } = renderHook(() => useDialogResize());

		expect(result.current.size).toEqual({ width: null, height: null });
		expect(result.current.isResizing).toBe(false);
		expect(result.current.resizeOffset).toEqual({ x: 0, y: 0 });
	});

	it("should expose all 8 resize directions", () => {
		const { result } = renderHook(() => useDialogResize());

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
		const { result } = renderHook(() => useDialogResize());

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
		const { result } = renderHook(() => useDialogResize());

		// Mock the popup element with getBoundingClientRect
		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		expect(result.current.resizeOffset).toEqual({ x: 0, y: 0 });
	});

	it("should resize south direction correctly", () => {
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		expect(result.current.resizeOffset.x).toBe(-50);
	});

	it("should resize north direction with offset adjustment", () => {
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		expect(result.current.resizeOffset.y).toBe(-50);
	});

	it("should resize se corner (both width and height)", () => {
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		expect(result.current.resizeOffset).toEqual({ x: 0, y: 0 });
	});

	it("should enforce minimum width constraint", () => {
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 400,
			height: 400,
			top: 0,
			left: 0,
			right: 400,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 300,
			top: 0,
			left: 0,
			right: 500,
			bottom: 300,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		const { result } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
		const { result, unmount } = renderHook(() => useDialogResize());

		const mockEl = document.createElement("div");
		vi.spyOn(mockEl, "getBoundingClientRect").mockReturnValue({
			width: 500,
			height: 400,
			top: 0,
			left: 0,
			right: 500,
			bottom: 400,
			x: 0,
			y: 0,
			toJSON: vi.fn(),
		});
		result.current.popupRef.current = mockEl;

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
});
