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
import type { Settings } from "@/modules/settings/hooks/useSettings";

interface AppearanceCardProps {
	settings: Settings;
	onThemeChange: (value: Settings["theme"] | null) => void;
}

export function AppearanceCard({
	settings,
	onThemeChange,
}: AppearanceCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Appearance</CardTitle>
				<CardDescription>Customize how the app looks</CardDescription>
			</CardHeader>
			<CardContent>
				<div className="flex items-center justify-between">
					<Label htmlFor="theme">Theme</Label>
					<Select value={settings.theme} onValueChange={onThemeChange}>
						<SelectTrigger className="w-32">
							<SelectValue />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="light">Light</SelectItem>
							<SelectItem value="dark">Dark</SelectItem>
							<SelectItem value="system">System</SelectItem>
						</SelectContent>
					</Select>
				</div>
			</CardContent>
		</Card>
	);
}
