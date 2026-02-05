import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
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

interface LoggingCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function LoggingCard({ settings, onUpdate }: LoggingCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Logging</CardTitle>
				<CardDescription>Configure application logging</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				<div className="flex items-center justify-between">
					<Label htmlFor="enable-logging">Enable logging</Label>
					<Switch
						id="enable-logging"
						checked={settings.enable_logging}
						onCheckedChange={(checked) => onUpdate({ enable_logging: checked })}
					/>
				</div>
				<div className="flex items-center justify-between">
					<Label htmlFor="log-level">Log level</Label>
					<Select
						value={settings.log_level}
						onValueChange={(value) => onUpdate({ log_level: value })}
						disabled={!settings.enable_logging}
					>
						<SelectTrigger className="w-32">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="error">Error</SelectItem>
							<SelectItem value="warn">Warn</SelectItem>
							<SelectItem value="info">Info</SelectItem>
							<SelectItem value="debug">Debug</SelectItem>
							<SelectItem value="trace">Trace</SelectItem>
						</SelectContent>
					</Select>
				</div>
			</CardContent>
		</Card>
	);
}
