import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { COMMANDS, UI } from "@/constants";
import {
	type Settings,
	type UpdateSettingsInput,
	useSettings,
} from "@/modules/settings/hooks/useSettings";
import { useTheme } from "@/modules/shared/providers/theme-provider";

interface LogEntry {
	timestamp: Date;
	success: boolean;
	message: string;
}

export function TestSettingsComponent() {
	const { settings, isLoading, updateSettingsAsync } = useSettings();
	const { theme: domTheme, setTheme } = useTheme();
	const [log, setLog] = useState<LogEntry[]>([]);

	const addLog = (success: boolean, message: string) => {
		setLog((prev) => [{ timestamp: new Date(), success, message }, ...prev]);
	};

	const formatTime = (date: Date) => {
		return date.toLocaleTimeString("en-US", { hour12: false });
	};

	const handleThemeChange = async (value: Settings["theme"]) => {
		try {
			setTheme(value);
			await updateSettingsAsync({ theme: value });
			addLog(true, `Theme set to ${value}`);
			toast.success(`Theme set to ${value}`);
		} catch {
			addLog(false, `Failed to set theme to ${value}`);
			toast.error("Failed to update theme");
		}
	};

	const handleToggle = async (
		key: keyof Omit<Settings, "theme" | "log_level" | "updated_at">,
	) => {
		if (!settings) return;
		const currentValue = settings[key];
		try {
			await updateSettingsAsync({
				[key]: !currentValue,
			} as UpdateSettingsInput);
			addLog(true, `${key} toggled to ${!currentValue}`);
			toast.success(`${key} updated`);
		} catch {
			addLog(false, `Failed to toggle ${key}`);
			toast.error("Update failed");
		}
	};

	const handleLogLevel = async (level: string) => {
		try {
			await updateSettingsAsync({ log_level: level });
			addLog(true, `Log level set to ${level}`);
			toast.success(`Log level set to ${level}`);
		} catch {
			addLog(false, `Failed to set log level to ${level}`);
			toast.error("Failed to update log level");
		}
	};

	const clearLog = () => {
		setLog([]);
	};

	const handleTestNotification = async () => {
		try {
			const sent = await invoke<boolean>(COMMANDS.SEND_NOTIFICATION, {
				input: {
					title: "Test Notification",
					body: "This is a test notification from Product Stalker!",
				},
			});
			if (sent) {
				addLog(true, "Test notification sent");
				toast.success("Notification sent");
			} else {
				addLog(false, "Notification skipped (notifications disabled)");
				toast.info("Notifications are disabled");
			}
		} catch {
			addLog(false, "Failed to send notification");
			toast.error("Failed to send notification");
		}
	};

	if (isLoading) {
		return (
			<div className="container mx-auto max-w-3xl px-4 py-6">
				<p className="text-muted-foreground">Loading settings...</p>
			</div>
		);
	}

	if (!settings) {
		return (
			<div className="container mx-auto max-w-3xl px-4 py-6">
				<p className="text-muted-foreground">Failed to load settings</p>
			</div>
		);
	}

	const booleanSettings: Array<
		keyof Omit<Settings, "theme" | "log_level" | "updated_at">
	> = [
		"show_in_tray",
		"launch_at_login",
		"enable_logging",
		"enable_notifications",
		"sidebar_expanded",
	];

	const logLevels = UI.LOG_LEVELS;

	return (
		<div className="container mx-auto max-w-3xl overflow-y-auto px-4 py-6">
			<h1 className="mb-6 font-semibold text-xl">Settings Feature Tests</h1>

			<div className="space-y-4">
				{/* Current Settings State */}
				<Card>
					<CardHeader>
						<CardTitle>Current Settings State</CardTitle>
					</CardHeader>
					<CardContent className="space-y-3">
						<pre className="max-h-48 overflow-auto rounded bg-muted p-3 text-xs">
							{JSON.stringify(settings, null, 2)}
						</pre>
						<Button
							variant="outline"
							size="sm"
							onClick={() => window.location.reload()}
						>
							Refresh Settings
						</Button>
					</CardContent>
				</Card>

				{/* Theme Tests */}
				<Card>
					<CardHeader>
						<CardTitle>Theme Tests</CardTitle>
					</CardHeader>
					<CardContent className="space-y-3">
						<div className="flex flex-wrap gap-2">
							<Button
								variant={settings.theme === "light" ? "default" : "outline"}
								size="sm"
								onClick={() => handleThemeChange("light")}
							>
								Set Light
							</Button>
							<Button
								variant={settings.theme === "dark" ? "default" : "outline"}
								size="sm"
								onClick={() => handleThemeChange("dark")}
							>
								Set Dark
							</Button>
							<Button
								variant={settings.theme === "system" ? "default" : "outline"}
								size="sm"
								onClick={() => handleThemeChange("system")}
							>
								Set System
							</Button>
						</div>
						<p className="text-muted-foreground text-sm">
							Current DOM theme: <span className="font-mono">{domTheme}</span>
						</p>
					</CardContent>
				</Card>

				{/* Boolean Settings Tests */}
				<Card>
					<CardHeader>
						<CardTitle>Boolean Settings Tests</CardTitle>
					</CardHeader>
					<CardContent>
						<div className="space-y-2">
							{booleanSettings.map((key) => (
								<div key={key} className="flex items-center justify-between">
									<Button
										variant="outline"
										size="sm"
										onClick={() => handleToggle(key)}
									>
										Toggle {key}
									</Button>
									<span className="text-sm">
										Current: {settings[key] ? "\u2713" : "\u2717"}
									</span>
								</div>
							))}
						</div>
					</CardContent>
				</Card>

				{/* Log Level Tests */}
				<Card>
					<CardHeader>
						<CardTitle>Log Level Tests</CardTitle>
					</CardHeader>
					<CardContent className="space-y-3">
						<div className="flex flex-wrap gap-2">
							{logLevels.map((level) => (
								<Button
									key={level}
									variant={settings.log_level === level ? "default" : "outline"}
									size="sm"
									onClick={() => handleLogLevel(level)}
								>
									{level}
								</Button>
							))}
						</div>
						<p className="text-muted-foreground text-sm">
							Current: <span className="font-mono">{settings.log_level}</span>
						</p>
					</CardContent>
				</Card>

				{/* Notification Tests */}
				<Card>
					<CardHeader>
						<CardTitle>Notification Tests</CardTitle>
					</CardHeader>
					<CardContent className="space-y-3">
						<Button
							variant="outline"
							size="sm"
							onClick={handleTestNotification}
						>
							Send Test Notification
						</Button>
						<p className="text-muted-foreground text-sm">
							Notifications enabled:{" "}
							{settings.enable_notifications ? "\u2713" : "\u2717"}
						</p>
					</CardContent>
				</Card>

				{/* Test Results Log */}
				<Card>
					<CardHeader>
						<CardTitle>Test Results Log</CardTitle>
					</CardHeader>
					<CardContent className="space-y-3">
						<div className="max-h-48 overflow-auto rounded bg-muted p-3 font-mono text-xs">
							{log.length === 0 ? (
								<span className="text-muted-foreground">
									No test results yet
								</span>
							) : (
								log.map((entry) => (
									<div key={entry.timestamp.getTime()}>
										[{formatTime(entry.timestamp)}]{" "}
										{entry.success ? "\u2713" : "\u2717"} {entry.message}
									</div>
								))
							)}
						</div>
						<Button variant="outline" size="sm" onClick={clearLog}>
							Clear Log
						</Button>
					</CardContent>
				</Card>
			</div>
		</div>
	);
}
