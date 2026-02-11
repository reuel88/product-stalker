import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";
import { SettingsSwitchRow } from "./settings-switch-row";

interface AdvancedCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function AdvancedCard({ settings, onUpdate }: AdvancedCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Advanced</CardTitle>
				<CardDescription>
					Advanced settings for compatibility with protected sites
				</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				<SettingsSwitchRow
					id="headless-browser"
					label="Headless browser"
					description="Use Chrome to check sites with bot protection (Cloudflare)"
					checked={settings.enable_headless_browser}
					onCheckedChange={(checked) =>
						onUpdate({ enable_headless_browser: checked })
					}
				/>
			</CardContent>
		</Card>
	);
}
