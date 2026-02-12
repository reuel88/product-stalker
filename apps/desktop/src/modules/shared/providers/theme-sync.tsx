import { useEffect, useRef } from "react";
import { useSettings } from "@/modules/settings/hooks/useSettings";
import { useTheme } from "./theme-provider";

export function ThemeSync() {
	const { settings } = useSettings();
	const { setTheme, theme } = useTheme();
	const initializedFromBackend = useRef(false);

	// Sync theme from backend settings on first load
	useEffect(() => {
		if (initializedFromBackend.current || !settings?.theme) return;
		initializedFromBackend.current = true;

		if (settings.theme !== theme) {
			setTheme(settings.theme);
		}
	}, [settings?.theme, theme, setTheme]);

	return null;
}
