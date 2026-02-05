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
				<div className="flex items-center justify-between">
					<Label htmlFor="enable-notifications">Enable notifications</Label>
					<Switch
						id="enable-notifications"
						checked={settings.enable_notifications}
						onCheckedChange={(checked) =>
							onUpdate({ enable_notifications: checked })
						}
					/>
				</div>
			</CardContent>
		</Card>
	);
}
