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
				<div className="flex items-center justify-between">
					<Label htmlFor="sidebar-expanded">Sidebar expanded</Label>
					<Switch
						id="sidebar-expanded"
						checked={settings.sidebar_expanded}
						onCheckedChange={(checked) =>
							onUpdate({ sidebar_expanded: checked })
						}
					/>
				</div>
			</CardContent>
		</Card>
	);
}
