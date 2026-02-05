import { toast } from "sonner";
import { MESSAGES } from "@/constants";
import { withToast } from "@/lib/toast-helpers";
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
import { FullPageError } from "@/modules/shared/ui/components/full-page-error";

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
		await withToast(() => updateSettingsAsync(input), {
			success: MESSAGES.SETTINGS.SAVED,
			error: MESSAGES.SETTINGS.SAVE_FAILED,
		});
	};

	const handleThemeChange = async (value: Settings["theme"]) => {
		setTheme(value);
		await handleUpdate({ theme: value });
	};

	const handleCheckForUpdate = async () => {
		try {
			const info = await checkForUpdateAsync();
			if (info.available && info.version) {
				toast.success(MESSAGES.UPDATE.AVAILABLE(info.version));
			} else {
				toast.info(MESSAGES.UPDATE.LATEST);
			}
		} catch {
			toast.error(MESSAGES.UPDATE.CHECK_FAILED);
		}
	};

	const handleInstallUpdate = async () => {
		try {
			toast.info(MESSAGES.UPDATE.DOWNLOADING);
			await installUpdateAsync();
		} catch {
			toast.error(MESSAGES.UPDATE.INSTALL_FAILED);
		}
	};

	if (isLoading) {
		return <SettingsSkeleton />;
	}

	if (!settings) {
		return (
			<FullPageError
				title="Failed to load settings"
				description="Please try again later"
			/>
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
