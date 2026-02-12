import type { ReactNode } from "react";
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
	useState,
} from "react";
import {
	applyPalette,
	clearPaletteStyles,
	detectCurrentMode,
	getStoredPaletteId,
	storePaletteId,
} from "@/modules/shared/themes/apply-palette";
import { DEFAULT_PALETTE_ID } from "@/modules/shared/themes/constants";
import { isValidPaletteId, PALETTES } from "@/modules/shared/themes/palettes";
import type {
	PaletteDefinition,
	PaletteId,
} from "@/modules/shared/themes/types";
import { useTheme } from "./theme-provider";

function applyOrClearPalette(id: PaletteId, mode: "light" | "dark") {
	if (id === DEFAULT_PALETTE_ID) {
		clearPaletteStyles();
	} else {
		applyPalette(id, mode);
	}
}

interface PaletteContextValue {
	paletteId: PaletteId;
	setPalette: (id: PaletteId) => void;
	palettes: PaletteDefinition[];
}

const PaletteContext = createContext<PaletteContextValue | null>(null);

interface PaletteProviderProps {
	children: ReactNode;
	/** Backend color palette value to sync from on first load */
	backendColorPalette?: string;
	/** Callback to persist palette changes to the backend */
	onPaletteChange?: (id: PaletteId) => void;
}

export function PaletteProvider({
	children,
	backendColorPalette,
	onPaletteChange,
}: PaletteProviderProps) {
	const [paletteId, setPaletteId] = useState<PaletteId>(getStoredPaletteId);
	const { resolvedTheme } = useTheme();
	const initializedFromBackend = useRef(false);

	// Sync palette from backend settings on first load
	useEffect(() => {
		if (initializedFromBackend.current || !backendColorPalette) return;
		initializedFromBackend.current = true;

		if (
			isValidPaletteId(backendColorPalette) &&
			backendColorPalette !== paletteId
		) {
			setPaletteId(backendColorPalette);
			storePaletteId(backendColorPalette);
		}
	}, [backendColorPalette, paletteId]);

	// Apply palette whenever paletteId or resolved theme changes
	useEffect(() => {
		const mode = resolvedTheme === "dark" ? "dark" : "light";
		applyOrClearPalette(paletteId, mode);
	}, [paletteId, resolvedTheme]);

	// Also listen for class changes on <html> to catch system theme transitions
	useEffect(() => {
		const observer = new MutationObserver(() => {
			applyOrClearPalette(paletteId, detectCurrentMode());
		});

		observer.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ["class"],
		});

		return () => observer.disconnect();
	}, [paletteId]);

	const setPalette = useCallback(
		(id: PaletteId) => {
			setPaletteId(id);
			storePaletteId(id);
			applyOrClearPalette(id, detectCurrentMode());
			onPaletteChange?.(id);
		},
		[onPaletteChange],
	);

	return (
		<PaletteContext.Provider
			value={{ paletteId, setPalette, palettes: PALETTES }}
		>
			{children}
		</PaletteContext.Provider>
	);
}

export function usePalette(): PaletteContextValue {
	const context = useContext(PaletteContext);
	if (!context) {
		throw new Error("usePalette must be used within a PaletteProvider");
	}
	return context;
}
