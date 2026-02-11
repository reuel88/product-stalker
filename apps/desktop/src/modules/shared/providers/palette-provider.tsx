import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useRef,
	useState,
} from "react";
import { useSettings } from "@/modules/settings/hooks/useSettings";
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

interface PaletteContextValue {
	paletteId: PaletteId;
	setPalette: (id: PaletteId) => void;
	palettes: PaletteDefinition[];
}

const PaletteContext = createContext<PaletteContextValue | null>(null);

export function PaletteProvider({ children }: { children: React.ReactNode }) {
	const [paletteId, setPaletteId] = useState<PaletteId>(getStoredPaletteId);
	const { resolvedTheme } = useTheme();
	const { settings, updateSettings } = useSettings();
	const initializedFromBackend = useRef(false);

	// Sync palette from backend settings on first load
	useEffect(() => {
		if (initializedFromBackend.current || !settings) return;
		initializedFromBackend.current = true;

		const backendPalette = settings.color_palette;
		if (
			backendPalette &&
			isValidPaletteId(backendPalette) &&
			backendPalette !== paletteId
		) {
			setPaletteId(backendPalette);
			storePaletteId(backendPalette);
		}
	}, [settings, paletteId]);

	// Apply palette whenever paletteId or resolved theme changes
	useEffect(() => {
		const mode = resolvedTheme === "dark" ? "dark" : "light";
		if (paletteId === DEFAULT_PALETTE_ID) {
			clearPaletteStyles();
		} else {
			applyPalette(paletteId, mode);
		}
	}, [paletteId, resolvedTheme]);

	// Also listen for class changes on <html> to catch system theme transitions
	useEffect(() => {
		const observer = new MutationObserver(() => {
			const mode = detectCurrentMode();
			if (paletteId === DEFAULT_PALETTE_ID) {
				clearPaletteStyles();
			} else {
				applyPalette(paletteId, mode);
			}
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

			const mode = detectCurrentMode();
			if (id === DEFAULT_PALETTE_ID) {
				clearPaletteStyles();
			} else {
				applyPalette(id, mode);
			}

			updateSettings({ color_palette: id });
		},
		[updateSettings],
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
