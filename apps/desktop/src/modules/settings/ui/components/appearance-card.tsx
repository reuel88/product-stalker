import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import type { Settings } from "@/modules/settings/hooks/useSettings";
import { usePalette } from "@/modules/shared/providers/palette-provider";
import type {
	PaletteDefinition,
	PaletteId,
} from "@/modules/shared/themes/types";
import { SettingsCard } from "./settings-card";

interface AppearanceCardProps {
	settings: Settings;
	onThemeChange: (value: Settings["theme"]) => void;
}

function PaletteCard({
	palette,
	isActive,
	onSelect,
}: {
	palette: PaletteDefinition;
	isActive: boolean;
	onSelect: (id: PaletteId) => void;
}) {
	return (
		<button
			type="button"
			onClick={() => onSelect(palette.id)}
			className={cn(
				"flex flex-col items-center gap-2 rounded-lg border-2 p-3 transition-colors",
				isActive
					? "border-primary bg-primary/5"
					: "border-border hover:border-primary/50",
			)}
		>
			<div className="flex gap-1">
				<div
					className="h-6 w-6 rounded-full border border-border"
					style={{ backgroundColor: palette.preview.primary }}
				/>
				<div
					className="h-6 w-6 rounded-full border border-border"
					style={{ backgroundColor: palette.preview.accent }}
				/>
				<div
					className="h-6 w-6 rounded-full border border-border"
					style={{ backgroundColor: palette.preview.background }}
				/>
			</div>
			<span className="font-medium text-xs">{palette.name}</span>
		</button>
	);
}

export function AppearanceCard({
	settings,
	onThemeChange,
}: AppearanceCardProps) {
	const { paletteId, setPalette, palettes } = usePalette();

	return (
		<SettingsCard
			title="Appearance"
			description="Customize how the app looks"
			contentClassName="space-y-4"
		>
			<div className="flex items-center justify-between">
				<Label htmlFor="theme">Theme</Label>
				<Select value={settings.theme} onValueChange={onThemeChange}>
					<SelectTrigger className="w-32">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="light">Light</SelectItem>
						<SelectItem value="dark">Dark</SelectItem>
						<SelectItem value="system">System</SelectItem>
					</SelectContent>
				</Select>
			</div>
			<div className="space-y-2">
				<Label>Color Palette</Label>
				<div className="grid grid-cols-3 gap-2">
					{palettes.map((palette) => (
						<PaletteCard
							key={palette.id}
							palette={palette}
							isActive={palette.id === paletteId}
							onSelect={setPalette}
						/>
					))}
				</div>
			</div>
		</SettingsCard>
	);
}
