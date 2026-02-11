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

interface InterfaceCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function InterfaceCard({ settings, onUpdate }: InterfaceCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Interface</CardTitle>
				<CardDescription>UI preferences</CardDescription>
			</CardHeader>
			<CardContent>
				<SettingsSwitchRow
					id="sidebar-expanded"
					label="Sidebar expanded"
					checked={settings.sidebar_expanded}
					onCheckedChange={(checked) => onUpdate({ sidebar_expanded: checked })}
				/>
			</CardContent>
		</Card>
	);
}
