import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import type {
	Settings,
	UpdateSettingsInput,
} from "@/modules/settings/hooks/useSettings";
import { SettingsCard } from "./settings-card";

interface DateDisplayCardProps {
	settings: Settings;
	onUpdate: (input: UpdateSettingsInput) => void;
}

/**
 * Curated list of common IANA timezones covering major regions
 */
const TIMEZONE_OPTIONS = [
	{ value: "auto", label: "Auto (System)" },
	{ value: "America/New_York", label: "Eastern Time (US)" },
	{ value: "America/Chicago", label: "Central Time (US)" },
	{ value: "America/Denver", label: "Mountain Time (US)" },
	{ value: "America/Los_Angeles", label: "Pacific Time (US)" },
	{ value: "America/Anchorage", label: "Alaska Time (US)" },
	{ value: "Pacific/Honolulu", label: "Hawaii Time (US)" },
	{ value: "America/Toronto", label: "Toronto" },
	{ value: "America/Vancouver", label: "Vancouver" },
	{ value: "America/Mexico_City", label: "Mexico City" },
	{ value: "America/Sao_Paulo", label: "São Paulo" },
	{ value: "America/Buenos_Aires", label: "Buenos Aires" },
	{ value: "Europe/London", label: "London" },
	{ value: "Europe/Paris", label: "Paris" },
	{ value: "Europe/Berlin", label: "Berlin" },
	{ value: "Europe/Madrid", label: "Madrid" },
	{ value: "Europe/Rome", label: "Rome" },
	{ value: "Europe/Amsterdam", label: "Amsterdam" },
	{ value: "Europe/Stockholm", label: "Stockholm" },
	{ value: "Europe/Moscow", label: "Moscow" },
	{ value: "Asia/Dubai", label: "Dubai" },
	{ value: "Asia/Kolkata", label: "India" },
	{ value: "Asia/Shanghai", label: "China" },
	{ value: "Asia/Tokyo", label: "Tokyo" },
	{ value: "Asia/Seoul", label: "Seoul" },
	{ value: "Asia/Singapore", label: "Singapore" },
	{ value: "Asia/Hong_Kong", label: "Hong Kong" },
	{ value: "Australia/Sydney", label: "Sydney" },
	{ value: "Australia/Melbourne", label: "Melbourne" },
	{ value: "Pacific/Auckland", label: "Auckland" },
] as const;

const DATE_FORMAT_OPTIONS = [
	{ value: "system", label: "System Default" },
	{ value: "MM/DD/YYYY", label: "MM/DD/YYYY (US)" },
	{ value: "DD/MM/YYYY", label: "DD/MM/YYYY (EU)" },
	{ value: "YYYY-MM-DD", label: "YYYY-MM-DD (ISO)" },
] as const;

export function DateDisplayCard({ settings, onUpdate }: DateDisplayCardProps) {
	return (
		<SettingsCard
			title="Date & Time Display"
			description="Configure how dates and times are displayed"
			contentClassName="space-y-4"
		>
			<div className="flex items-center justify-between">
				<Label htmlFor="timezone">Timezone</Label>
				<Select
					value={settings.display_timezone}
					onValueChange={(value) => onUpdate({ display_timezone: value })}
				>
					<SelectTrigger id="timezone" className="w-48">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{TIMEZONE_OPTIONS.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
			<div className="flex items-center justify-between">
				<Label htmlFor="date-format">Date Format</Label>
				<Select
					value={settings.date_format}
					onValueChange={(value) => onUpdate({ date_format: value })}
				>
					<SelectTrigger id="date-format" className="w-48">
						<SelectValue />
					</SelectTrigger>
					<SelectContent>
						{DATE_FORMAT_OPTIONS.map((option) => (
							<SelectItem key={option.value} value={option.value}>
								{option.label}
							</SelectItem>
						))}
					</SelectContent>
				</Select>
			</div>
		</SettingsCard>
	);
}
