import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";

interface SettingsCardProps {
	/** The card title displayed in the header */
	title: string;
	/** The card description displayed in the header */
	description: string;
	/** The content to display inside the card */
	children: React.ReactNode;
	/** Optional className for the CardContent (e.g., "space-y-4" for vertical spacing) */
	contentClassName?: string;
}

/**
 * Base component for settings cards.
 *
 * Provides consistent structure for settings sections with a title, description,
 * and content area. Eliminates boilerplate Card/CardHeader/CardTitle/CardDescription wrapping.
 *
 * @example
 * ```tsx
 * <SettingsCard
 *   title="Notifications"
 *   description="Configure notification preferences"
 * >
 *   <SettingsSwitchRow
 *     id="enable-notifications"
 *     label="Enable notifications"
 *     checked={settings.enable_notifications}
 *     onCheckedChange={(checked) => onUpdate({ enable_notifications: checked })}
 *   />
 * </SettingsCard>
 * ```
 */
export function SettingsCard({
	title,
	description,
	children,
	contentClassName,
}: SettingsCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>{title}</CardTitle>
				<CardDescription>{description}</CardDescription>
			</CardHeader>
			<CardContent className={contentClassName}>{children}</CardContent>
		</Card>
	);
}
