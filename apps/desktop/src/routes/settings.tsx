import { createFileRoute } from "@tanstack/react-router";
import { toast } from "sonner";

import { useTheme } from "@/components/theme-provider";
import { Button } from "@/components/ui/button";
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
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { MESSAGES } from "@/constants";
import {
	type Settings,
	type UpdateSettingsInput,
	useSettings,
} from "@/hooks/useSettings";
import { useUpdater } from "@/hooks/useUpdater";

export const Route = createFileRoute("/settings")({
	component: SettingsComponent,
});

function SettingsComponent() {
	const { settings, isLoading, updateSettingsAsync } = useSettings();
	const { setTheme } = useTheme();
	const {
		currentVersion,
		updateInfo,
		isChecking,
		isInstalling,
		checkForUpdateAsync,
		installUpdateAsync,
	} = useUpdater();

	const handleUpdate = async (input: UpdateSettingsInput) => {
		try {
			await updateSettingsAsync(input);
			toast.success(MESSAGES.SETTINGS.SAVED);
		} catch {
			toast.error(MESSAGES.SETTINGS.SAVE_FAILED);
		}
	};

	const handleThemeChange = async (value: Settings["theme"] | null) => {
		if (!value) return;
		setTheme(value);
		await handleUpdate({ theme: value });
	};

	const handleCheckForUpdate = async () => {
		try {
			const info = await checkForUpdateAsync();
			if (info.available) {
				toast.success(`Update available: v${info.version}`);
			} else {
				toast.info("You're running the latest version");
			}
		} catch {
			toast.error("Failed to check for updates");
		}
	};

	const handleInstallUpdate = async () => {
		try {
			toast.info("Downloading update...");
			await installUpdateAsync();
		} catch {
			toast.error("Failed to install update");
		}
	};

	if (isLoading) {
		return <SettingsSkeleton />;
	}

	if (!settings) {
		return (
			<div className="container mx-auto max-w-2xl px-4 py-6">
				<p className="text-muted-foreground">Failed to load settings</p>
			</div>
		);
	}

	return (
		<div className="container mx-auto max-w-2xl overflow-y-auto px-4 py-6">
			<h1 className="mb-6 font-semibold text-xl">Settings</h1>

			<div className="space-y-4">
				{/* Appearance */}
				<Card>
					<CardHeader>
						<CardTitle>Appearance</CardTitle>
						<CardDescription>Customize how the app looks</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="flex items-center justify-between">
							<Label htmlFor="theme">Theme</Label>
							<Select value={settings.theme} onValueChange={handleThemeChange}>
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

				{/* System */}
				<Card>
					<CardHeader>
						<CardTitle>System</CardTitle>
						<CardDescription>System integration settings</CardDescription>
					</CardHeader>
					<CardContent className="space-y-4">
						<div className="flex items-center justify-between">
							<Label htmlFor="show-in-tray">Show in tray</Label>
							<Switch
								id="show-in-tray"
								checked={settings.show_in_tray}
								onCheckedChange={(checked) =>
									handleUpdate({ show_in_tray: checked })
								}
							/>
						</div>
						<div className="flex items-center justify-between">
							<Label htmlFor="launch-at-login">Launch at login</Label>
							<Switch
								id="launch-at-login"
								checked={settings.launch_at_login}
								onCheckedChange={(checked) =>
									handleUpdate({ launch_at_login: checked })
								}
							/>
						</div>
					</CardContent>
				</Card>

				{/* Logging */}
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
								onCheckedChange={(checked) =>
									handleUpdate({ enable_logging: checked })
								}
							/>
						</div>
						<div className="flex items-center justify-between">
							<Label htmlFor="log-level">Log level</Label>
							<Select
								value={settings.log_level}
								onValueChange={(value: string | null) =>
									value && handleUpdate({ log_level: value })
								}
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

				{/* Notifications */}
				<Card>
					<CardHeader>
						<CardTitle>Notifications</CardTitle>
						<CardDescription>
							Configure notification preferences
						</CardDescription>
					</CardHeader>
					<CardContent>
						<div className="flex items-center justify-between">
							<Label htmlFor="enable-notifications">Enable notifications</Label>
							<Switch
								id="enable-notifications"
								checked={settings.enable_notifications}
								onCheckedChange={(checked) =>
									handleUpdate({ enable_notifications: checked })
								}
							/>
						</div>
					</CardContent>
				</Card>

				{/* Interface */}
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
									handleUpdate({ sidebar_expanded: checked })
								}
							/>
						</div>
					</CardContent>
				</Card>

				{/* Updates */}
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
								onClick={handleCheckForUpdate}
								disabled={isChecking || isInstalling}
							>
								{isChecking ? "Checking..." : "Check for Updates"}
							</Button>
							{updateInfo?.available && (
								<Button
									size="sm"
									onClick={handleInstallUpdate}
									disabled={isInstalling}
								>
									{isInstalling ? "Installing..." : "Update Now"}
								</Button>
							)}
						</div>
					</CardContent>
				</Card>
			</div>
		</div>
	);
}

function SettingsSkeleton() {
	return (
		<div className="container mx-auto max-w-2xl px-4 py-6">
			<Skeleton className="mb-6 h-7 w-24" />
			<div className="space-y-4">
				{[1, 2, 3, 4, 5].map((i) => (
					<Card key={i}>
						<CardHeader>
							<Skeleton className="h-5 w-32" />
							<Skeleton className="h-4 w-48" />
						</CardHeader>
						<CardContent>
							<div className="flex items-center justify-between">
								<Skeleton className="h-4 w-24" />
								<Skeleton className="h-5 w-9 rounded-full" />
							</div>
						</CardContent>
					</Card>
				))}
			</div>
		</div>
	);
}
