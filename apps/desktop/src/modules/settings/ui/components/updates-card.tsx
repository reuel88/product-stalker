import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import type { UpdateInfo } from "@/modules/settings/hooks/useUpdater";

interface UpdatesCardProps {
	currentVersion: string | undefined;
	updateInfo: UpdateInfo | undefined;
	isChecking: boolean;
	isInstalling: boolean;
	onCheckForUpdate: () => void;
	onInstallUpdate: () => void;
}

export function UpdatesCard({
	currentVersion,
	updateInfo,
	isChecking,
	isInstalling,
	onCheckForUpdate,
	onInstallUpdate,
}: UpdatesCardProps) {
	return (
		<Card>
			<CardHeader>
				<CardTitle>Updates</CardTitle>
				<CardDescription>Check for application updates</CardDescription>
			</CardHeader>
			<CardContent className="space-y-4">
				<div className="flex items-center justify-between">
					<Label>Current version</Label>
					<span className="text-muted-foreground text-sm">
						v{currentVersion ?? "..."}
					</span>
				</div>
				{updateInfo?.available && (
					<div className="flex items-center justify-between">
						<Label>Available version</Label>
						<span className="font-medium text-green-600 text-sm dark:text-green-400">
							v{updateInfo.version}
						</span>
					</div>
				)}
				<div className="flex gap-2">
					<Button
						variant="outline"
						size="sm"
						onClick={onCheckForUpdate}
						disabled={isChecking || isInstalling}
					>
						{isChecking ? "Checking..." : "Check for Updates"}
					</Button>
					{updateInfo?.available && (
						<Button size="sm" onClick={onInstallUpdate} disabled={isInstalling}>
							{isInstalling ? "Installing..." : "Update Now"}
						</Button>
					)}
				</div>
			</CardContent>
		</Card>
	);
}
