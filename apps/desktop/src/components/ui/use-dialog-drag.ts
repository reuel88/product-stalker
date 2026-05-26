import { useCallback, useEffect, useRef, useState } from "react";

interface Offset {
	x: number;
	y: number;
}

const INTERACTIVE_SELECTORS =
	"button, input, textarea, select, a, [role='button']";

export function useDialogDrag(
	popupRef: React.RefObject<HTMLDivElement | null>,
) {
	const [offset, setOffset] = useState<Offset>({ x: 0, y: 0 });
	const [isDragging, setIsDragging] = useState(false);

	const startPos = useRef<{ x: number; y: number } | null>(null);
	const startOffset = useRef<Offset>({ x: 0, y: 0 });
	const startRect = useRef<DOMRect | null>(null);

	useEffect(() => {
		if (!isDragging) return;

		function onPointerMove(e: PointerEvent) {
			if (!startPos.current) return;
			let dx = e.clientX - startPos.current.x;
			let dy = e.clientY - startPos.current.y;

			if (startRect.current) {
				const vw = window.innerWidth;
				const vh = window.innerHeight;
				const r = startRect.current;
				if (r.width <= vw) {
					const newLeft = r.left + dx;
					const newRight = r.right + dx;
					if (newLeft < 0) dx -= newLeft;
					else if (newRight > vw) dx -= newRight - vw;
				}
				if (r.height <= vh) {
					const newTop = r.top + dy;
					const newBottom = r.bottom + dy;
					if (newTop < 0) dy -= newTop;
					else if (newBottom > vh) dy -= newBottom - vh;
				}
			}

			setOffset({
				x: startOffset.current.x + dx,
				y: startOffset.current.y + dy,
			});
		}

		function onPointerUp() {
			setIsDragging(false);
			startPos.current = null;
		}

		document.addEventListener("pointermove", onPointerMove);
		document.addEventListener("pointerup", onPointerUp);
		document.addEventListener("pointercancel", onPointerUp);
		return () => {
			document.removeEventListener("pointermove", onPointerMove);
			document.removeEventListener("pointerup", onPointerUp);
			document.removeEventListener("pointercancel", onPointerUp);
		};
	}, [isDragging]);

	const handlePointerDown = useCallback(
		(e: React.PointerEvent) => {
			const target = e.target as HTMLElement;
			if (target.closest(INTERACTIVE_SELECTORS)) return;
			if (e.button !== 0 || !e.isPrimary) return;

			e.preventDefault();
			startPos.current = { x: e.clientX, y: e.clientY };
			startOffset.current = { ...offset };
			startRect.current = popupRef.current?.getBoundingClientRect() ?? null;
			setIsDragging(true);
		},
		[offset, popupRef],
	);

	const reset = useCallback(() => {
		setOffset({ x: 0, y: 0 });
		setIsDragging(false);
		startPos.current = null;
	}, []);

	return { offset, isDragging, handlePointerDown, reset };
}
