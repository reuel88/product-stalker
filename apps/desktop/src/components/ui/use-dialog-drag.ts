import { useCallback, useEffect, useRef, useState } from "react";

interface Offset {
	x: number;
	y: number;
}

const INTERACTIVE_SELECTORS =
	"button, input, textarea, select, a, [role='button']";

export function useDialogDrag() {
	const [offset, setOffset] = useState<Offset>({ x: 0, y: 0 });
	const [isDragging, setIsDragging] = useState(false);

	const startPos = useRef<{ x: number; y: number } | null>(null);
	const startOffset = useRef<Offset>({ x: 0, y: 0 });

	useEffect(() => {
		if (!isDragging) return;

		function onPointerMove(e: PointerEvent) {
			if (!startPos.current) return;
			const dx = e.clientX - startPos.current.x;
			const dy = e.clientY - startPos.current.y;
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
		return () => {
			document.removeEventListener("pointermove", onPointerMove);
			document.removeEventListener("pointerup", onPointerUp);
		};
	}, [isDragging]);

	const handlePointerDown = useCallback(
		(e: React.PointerEvent) => {
			const target = e.target as HTMLElement;
			if (target.closest(INTERACTIVE_SELECTORS)) return;

			e.preventDefault();
			startPos.current = { x: e.clientX, y: e.clientY };
			startOffset.current = { ...offset };
			setIsDragging(true);
		},
		[offset],
	);

	const reset = useCallback(() => {
		setOffset({ x: 0, y: 0 });
		setIsDragging(false);
		startPos.current = null;
	}, []);

	return { offset, isDragging, handlePointerDown, reset };
}
