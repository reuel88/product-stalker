import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

interface SettingsSwitchRowProps {
	id: string;
	label: string;
	description?: string;
	checked: boolean;
	onCheckedChange: (checked: boolean) => void;
}

export function SettingsSwitchRow({
	id,
	label,
	description,
	checked,
	onCheckedChange,
}: SettingsSwitchRowProps) {
	return (
		<div className="flex items-center justify-between">
			<div className="space-y-0.5">
				<Label htmlFor={id}>{label}</Label>
				{description && (
					<p className="text-muted-foreground text-xs">{description}</p>
				)}
			</div>
			<Switch id={id} checked={checked} onCheckedChange={onCheckedChange} />
		</div>
	);
}
