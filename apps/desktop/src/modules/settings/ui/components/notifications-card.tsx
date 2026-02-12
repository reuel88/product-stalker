import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";
import { SettingsCard } from "./settings-card";
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
		<SettingsCard
			title="Notifications"
			description="Configure notification preferences"
		>
			<SettingsSwitchRow
				id="enable-notifications"
				label="Enable notifications"
				checked={settings.enable_notifications}
				onCheckedChange={(checked) =>
					onUpdate({ enable_notifications: checked })
				}
			/>
		</SettingsCard>
	);
}
