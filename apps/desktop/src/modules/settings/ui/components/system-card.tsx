import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";

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
				<div className="flex items-center justify-between">
					<Label htmlFor="show-in-tray">Show in tray</Label>
					<Switch
						id="show-in-tray"
						checked={settings.show_in_tray}
						onCheckedChange={(checked) => onUpdate({ show_in_tray: checked })}
					/>
				</div>
				<div className="flex items-center justify-between">
					<Label htmlFor="launch-at-login">Launch at login</Label>
					<Switch
						id="launch-at-login"
						checked={settings.launch_at_login}
						onCheckedChange={(checked) =>
							onUpdate({ launch_at_login: checked })
						}
					/>
				</div>
			</CardContent>
		</Card>
	);
}
