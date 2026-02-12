import {
	createRootRouteWithContext,
	HeadContent,
	Outlet,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import type { ReactNode } from "react";
import { useCallback } from "react";
import { Toaster } from "@/components/ui/sonner";
import { useSettings } from "@/modules/settings/hooks/useSettings";
import { PaletteProvider } from "@/modules/shared/providers/palette-provider";
import { ThemeProvider } from "@/modules/shared/providers/theme-provider";
import { ThemeSync } from "@/modules/shared/providers/theme-sync";
import Header from "@/modules/shared/ui/components/header";

import "../index.css";

export type RouterAppContext = Record<string, never>;

export const Route = createRootRouteWithContext<RouterAppContext>()({
	component: RootComponent,
	head: () => ({
		meta: [
			{
				title: "product-stalker",
			},
			{
				name: "description",
				content: "product-stalker is a web application",
			},
		],
		links: [
			{
				rel: "icon",
				href: "/favicon.ico",
			},
		],
	}),
});

function PaletteProviderWithSettings({ children }: { children: ReactNode }) {
	const { settings, updateSettings } = useSettings();

	const handlePaletteChange = useCallback(
		(id: string) => {
			updateSettings({ color_palette: id });
		},
		[updateSettings],
	);

	return (
		<PaletteProvider
			backendColorPalette={settings?.color_palette}
			onPaletteChange={handlePaletteChange}
		>
			{children}
		</PaletteProvider>
	);
}

export function RootComponent() {
	return (
		<>
			<HeadContent />
			<ThemeProvider
				attribute="class"
				defaultTheme="dark"
				disableTransitionOnChange
				storageKey="vite-ui-theme"
			>
				<ThemeSync />
				<PaletteProviderWithSettings>
					<div className="grid h-svh grid-rows-[auto_1fr]">
						<Header />
						<Outlet />
					</div>
					<Toaster richColors />
				</PaletteProviderWithSettings>
			</ThemeProvider>
			{import.meta.env.DEV && <TanStackRouterDevtools position="bottom-left" />}
		</>
	);
}
