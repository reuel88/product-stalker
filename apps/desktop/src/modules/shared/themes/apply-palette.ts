import {
	DEFAULT_PALETTE_ID,
	PALETTE_DATA_ATTRIBUTE,
	PALETTE_STORAGE_KEY,
} from "./constants";
import { isValidPaletteId, PALETTE_MAP } from "./palettes";
import type { PaletteId } from "./types";

export function applyPalette(paletteId: PaletteId, mode: "light" | "dark") {
	const palette = PALETTE_MAP[paletteId];
	if (!palette) return;

	const vars = mode === "dark" ? palette.dark : palette.light;
	const root = document.documentElement;

	for (const [key, value] of Object.entries(vars)) {
		root.style.setProperty(key, value);
	}

	root.setAttribute(PALETTE_DATA_ATTRIBUTE, paletteId);
}

export function clearPaletteStyles() {
	const root = document.documentElement;
	const defaultPalette = PALETTE_MAP.default;
	for (const key of Object.keys(defaultPalette.light)) {
		root.style.removeProperty(key);
	}
	root.removeAttribute(PALETTE_DATA_ATTRIBUTE);
}

export function getStoredPaletteId(): PaletteId {
	try {
		const stored = localStorage.getItem(PALETTE_STORAGE_KEY);
		if (stored && isValidPaletteId(stored)) {
			return stored;
		}
	} catch {
		// localStorage unavailable
	}
	return DEFAULT_PALETTE_ID;
}

export function storePaletteId(id: PaletteId) {
	try {
		localStorage.setItem(PALETTE_STORAGE_KEY, id);
	} catch {
		// localStorage unavailable
	}
}

export function detectCurrentMode(): "light" | "dark" {
	return document.documentElement.classList.contains("dark") ? "dark" : "light";
}

export function earlyApplyPalette() {
	const paletteId = getStoredPaletteId();
	if (paletteId === DEFAULT_PALETTE_ID) return;

	const mode = detectCurrentMode();
	applyPalette(paletteId, mode);
}
