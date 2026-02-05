import { toast } from "sonner";
import { MESSAGES } from "@/constants";
import {
	type Settings,
	type UpdateSettingsInput,
	useSettings,
} from "@/modules/settings/hooks/useSettings";
import { useUpdater } from "@/modules/settings/hooks/useUpdater";
import { AdvancedCard } from "@/modules/settings/ui/components/advanced-card";
import { AppearanceCard } from "@/modules/settings/ui/components/appearance-card";
import { BackgroundCheckingCard } from "@/modules/settings/ui/components/background-checking-card";
import { InterfaceCard } from "@/modules/settings/ui/components/interface-card";
import { LoggingCard } from "@/modules/settings/ui/components/logging-card";
import { NotificationsCard } from "@/modules/settings/ui/components/notifications-card";
import { SettingsSkeleton } from "@/modules/settings/ui/components/settings-skeleton";
import { SystemCard } from "@/modules/settings/ui/components/system-card";
import { UpdatesCard } from "@/modules/settings/ui/components/updates-card";
import { useTheme } from "@/modules/shared/providers/theme-provider";
import { ErrorState } from "@/modules/shared/ui/components/error-state";

export function SettingsView() {
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
			<div className="flex h-screen w-full flex-col items-center justify-center">
				<ErrorState
					title="Failed to load settings"
					description="Please try again later"
				/>
			</div>
		);
	}

	return (
		<div className="container mx-auto max-w-2xl overflow-y-auto px-4 py-6">
			<h1 className="mb-6 font-semibold text-xl">Settings</h1>

			<div className="space-y-4">
				<AppearanceCard settings={settings} onThemeChange={handleThemeChange} />
				<SystemCard settings={settings} onUpdate={handleUpdate} />
				<LoggingCard settings={settings} onUpdate={handleUpdate} />
				<NotificationsCard settings={settings} onUpdate={handleUpdate} />
				<BackgroundCheckingCard settings={settings} onUpdate={handleUpdate} />
				<AdvancedCard settings={settings} onUpdate={handleUpdate} />
				<InterfaceCard settings={settings} onUpdate={handleUpdate} />
				<UpdatesCard
					currentVersion={currentVersion}
					updateInfo={updateInfo}
					isChecking={isChecking}
					isInstalling={isInstalling}
					onCheckForUpdate={handleCheckForUpdate}
					onInstallUpdate={handleInstallUpdate}
				/>
			</div>
		</div>
	);
}
