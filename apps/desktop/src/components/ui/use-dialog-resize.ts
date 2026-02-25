import { useCallback, useEffect, useRef, useState } from "react";

type ResizeDirection = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";

interface Size {
	width: number | null;
	height: number | null;
}

interface ResizeOffset {
	x: number;
	y: number;
}

const MIN_WIDTH = 320;
const MIN_HEIGHT = 180;

const HANDLE_SIZE = 8;

const CURSOR_MAP: Record<ResizeDirection, string> = {
	n: "ns-resize",
	s: "ns-resize",
	e: "ew-resize",
	w: "ew-resize",
	ne: "nesw-resize",
	sw: "nesw-resize",
	nw: "nwse-resize",
	se: "nwse-resize",
};

const HANDLE_STYLES: Record<ResizeDirection, React.CSSProperties> = {
	n: {
		top: -HANDLE_SIZE / 2,
		left: HANDLE_SIZE / 2,
		right: HANDLE_SIZE / 2,
		height: HANDLE_SIZE,
	},
	s: {
		bottom: -HANDLE_SIZE / 2,
		left: HANDLE_SIZE / 2,
		right: HANDLE_SIZE / 2,
		height: HANDLE_SIZE,
	},
	e: {
		top: HANDLE_SIZE / 2,
		bottom: HANDLE_SIZE / 2,
		right: -HANDLE_SIZE / 2,
		width: HANDLE_SIZE,
	},
	w: {
		top: HANDLE_SIZE / 2,
		bottom: HANDLE_SIZE / 2,
		left: -HANDLE_SIZE / 2,
		width: HANDLE_SIZE,
	},
	ne: {
		top: -HANDLE_SIZE / 2,
		right: -HANDLE_SIZE / 2,
		width: HANDLE_SIZE * 2,
		height: HANDLE_SIZE * 2,
	},
	nw: {
		top: -HANDLE_SIZE / 2,
		left: -HANDLE_SIZE / 2,
		width: HANDLE_SIZE * 2,
		height: HANDLE_SIZE * 2,
	},
	se: {
		bottom: -HANDLE_SIZE / 2,
		right: -HANDLE_SIZE / 2,
		width: HANDLE_SIZE * 2,
		height: HANDLE_SIZE * 2,
	},
	sw: {
		bottom: -HANDLE_SIZE / 2,
		left: -HANDLE_SIZE / 2,
		width: HANDLE_SIZE * 2,
		height: HANDLE_SIZE * 2,
	},
};

const DIRECTIONS: ResizeDirection[] = [
	"n",
	"s",
	"e",
	"w",
	"ne",
	"nw",
	"se",
	"sw",
];

export function useDialogResize() {
	const [size, setSize] = useState<Size>({ width: null, height: null });
	const [isResizing, setIsResizing] = useState(false);
	const [resizeOffset, setResizeOffset] = useState<ResizeOffset>({
		x: 0,
		y: 0,
	});

	const activeDirection = useRef<ResizeDirection | null>(null);
	const startPos = useRef<{ x: number; y: number } | null>(null);
	const startSize = useRef<Size>({ width: null, height: null });
	const startResizeOffset = useRef<ResizeOffset>({ x: 0, y: 0 });
	const popupRef = useRef<HTMLElement | null>(null);

	useEffect(() => {
		if (!isResizing) return;

		function onPointerMove(e: PointerEvent) {
			if (!startPos.current || !activeDirection.current) return;
			const dir = activeDirection.current;
			const dx = e.clientX - startPos.current.x;
			const dy = e.clientY - startPos.current.y;

			const prevW = startSize.current.width ?? 0;
			const prevH = startSize.current.height ?? 0;

			let newW = prevW;
			let newH = prevH;
			let offX = startResizeOffset.current.x;
			let offY = startResizeOffset.current.y;

			if (dir.includes("e")) {
				newW = Math.max(MIN_WIDTH, prevW + dx);
			}
			if (dir.includes("w")) {
				const proposed = prevW - dx;
				const clamped = Math.max(MIN_WIDTH, proposed);
				const actualDx = prevW - clamped;
				newW = clamped;
				offX = startResizeOffset.current.x + actualDx;
			}
			if (dir.includes("s")) {
				newH = Math.max(MIN_HEIGHT, prevH + dy);
			}
			if (dir.includes("n")) {
				const proposed = prevH - dy;
				const clamped = Math.max(MIN_HEIGHT, proposed);
				const actualDy = prevH - clamped;
				newH = clamped;
				offY = startResizeOffset.current.y + actualDy;
			}

			setSize({ width: newW, height: newH });
			setResizeOffset({ x: offX, y: offY });
		}

		function onPointerUp() {
			setIsResizing(false);
			activeDirection.current = null;
			startPos.current = null;
		}

		document.addEventListener("pointermove", onPointerMove);
		document.addEventListener("pointerup", onPointerUp);
		return () => {
			document.removeEventListener("pointermove", onPointerMove);
			document.removeEventListener("pointerup", onPointerUp);
		};
	}, [isResizing]);

	const getHandleProps = useCallback(
		(direction: ResizeDirection) => ({
			onPointerDown: (e: React.PointerEvent) => {
				e.preventDefault();
				e.stopPropagation();

				// Capture current popup size if this is the first resize
				if (size.width === null || size.height === null) {
					const el = popupRef.current;
					if (el) {
						const rect = el.getBoundingClientRect();
						startSize.current = { width: rect.width, height: rect.height };
						setSize({ width: rect.width, height: rect.height });
					}
				} else {
					startSize.current = { ...size };
				}

				activeDirection.current = direction;
				startPos.current = { x: e.clientX, y: e.clientY };
				startResizeOffset.current = { ...resizeOffset };
				setIsResizing(true);
			},
			style: {
				...HANDLE_STYLES[direction],
				position: "absolute" as const,
				cursor: CURSOR_MAP[direction],
				zIndex: 50,
			},
			"data-resize-handle": direction,
		}),
		[size, resizeOffset],
	);

	const reset = useCallback(() => {
		setSize({ width: null, height: null });
		setResizeOffset({ x: 0, y: 0 });
		setIsResizing(false);
		activeDirection.current = null;
		startPos.current = null;
	}, []);

	return {
		size,
		isResizing,
		getHandleProps,
		resizeOffset,
		reset,
		popupRef,
		directions: DIRECTIONS,
	};
}
