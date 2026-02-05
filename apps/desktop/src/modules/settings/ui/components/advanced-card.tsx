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

interface AdvancedCardProps {
	settings: Settings;
	onUpdate: (updates: UpdateSettingsInput) => void;
}

export function AdvancedCard({ settings, onUpdate }: AdvancedCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Advanced</CardTitle>
				<CardDescription>
					Advanced settings for compatibility with protected sites
				</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				<div className="flex items-center justify-between">
					<div className="space-y-0.5">
						<Label htmlFor="headless-browser">Headless browser</Label>
						<p className="text-muted-foreground text-xs">
							Use Chrome to check sites with bot protection (Cloudflare)
						</p>
					</div>
					<Switch
						id="headless-browser"
						checked={settings.enable_headless_browser}
						onCheckedChange={(checked) =>
							onUpdate({ enable_headless_browser: checked })
						}
					/>
				</div>
			</CardContent>
		</Card>
	);
}
