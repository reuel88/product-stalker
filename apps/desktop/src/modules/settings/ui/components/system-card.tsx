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

interface SystemCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function SystemCard({ settings, onUpdate }: SystemCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>System</CardTitle>
				<CardDescription>System integration settings</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				<SettingsSwitchRow
					id="show-in-tray"
					label="Show in tray"
					checked={settings.show_in_tray}
					onCheckedChange={(checked) => onUpdate({ show_in_tray: checked })}
				/>
				<SettingsSwitchRow
					id="launch-at-login"
					label="Launch at login"
					checked={settings.launch_at_login}
					onCheckedChange={(checked) => onUpdate({ launch_at_login: checked })}
				/>
			</CardContent>
		</Card>
	);
}
