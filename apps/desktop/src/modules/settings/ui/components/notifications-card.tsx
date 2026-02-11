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

interface NotificationsCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function NotificationsCard({
	settings,
	onUpdate,
}: NotificationsCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Notifications</CardTitle>
				<CardDescription>Configure notification preferences</CardDescription>
			</CardHeader>
			<CardContent>
				<SettingsSwitchRow
					id="enable-notifications"
					label="Enable notifications"
					checked={settings.enable_notifications}
					onCheckedChange={(checked) =>
						onUpdate({ enable_notifications: checked })
					}
				/>
			</CardContent>
		</Card>
	);
}
