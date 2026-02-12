import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";
import { SettingsCard } from "./settings-card";
import { SettingsSwitchRow } from "./settings-switch-row";

interface InterfaceCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function InterfaceCard({ settings, onUpdate }: InterfaceCardProps) {
	return (
		<SettingsCard title="Interface" description="UI preferences">
			<SettingsSwitchRow
				id="sidebar-expanded"
				label="Sidebar expanded"
				checked={settings.sidebar_expanded}
				onCheckedChange={(checked) => onUpdate({ sidebar_expanded: checked })}
			/>
		</SettingsCard>
	);
}
