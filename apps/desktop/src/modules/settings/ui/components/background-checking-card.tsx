import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";
import { SettingsCard } from "./settings-card";

interface BackgroundCheckingCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function BackgroundCheckingCard({
	settings,
	onUpdate,
}: BackgroundCheckingCardProps) {
	return (
		<SettingsCard
			title="Background Checking"
			description="Automatically check product availability periodically"
			contentClassName="space-y-4"
		>
			<div className="flex items-center justify-between">
				<Label htmlFor="background-check">Enable background checking</Label>
				<Switch
					id="background-check"
					checked={settings.background_check_enabled}
					onCheckedChange={(checked) =>
						onUpdate({ background_check_enabled: checked })
					}
				/>
			</div>
			<div className="flex items-center justify-between">
				<Label htmlFor="check-interval">Check interval</Label>
				<Select
					value={String(settings.background_check_interval_minutes)}
					onValueChange={(value) =>
						onUpdate({
							background_check_interval_minutes: Number.parseInt(value, 10),
						})
					}
					disabled={!settings.background_check_enabled}
				>
					<SelectTrigger className="w-32">
						<SelectValue placeholder="Select interval" />
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="15">15 minutes</SelectItem>
						<SelectItem value="30">30 minutes</SelectItem>
						<SelectItem value="60">1 hour</SelectItem>
						<SelectItem value="240">4 hours</SelectItem>
						<SelectItem value="1440">Daily</SelectItem>
					</SelectContent>
				</Select>
			</div>
		</SettingsCard>
	);
}
