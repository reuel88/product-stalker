import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";
import { SettingsCard } from "./settings-card";
import { SettingsSwitchRow } from "./settings-switch-row";

interface AdvancedCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function AdvancedCard({ settings, onUpdate }: AdvancedCardProps) {
	return (
		<SettingsCard
			title="Advanced"
			description="Advanced settings for compatibility with protected sites"
			contentClassName="space-y-4"
		>
			<SettingsSwitchRow
				id="headless-browser"
				label="Headless browser"
				description="Use Chrome to check sites with bot protection (Cloudflare)"
				checked={settings.enable_headless_browser}
				onCheckedChange={(checked) =>
					onUpdate({ enable_headless_browser: checked })
				}
			/>
		</SettingsCard>
	);
}
