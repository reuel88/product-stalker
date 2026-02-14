import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";

/**
 * Settings interface - uses snake_case to match Rust backend serde serialization.
 *
 * The property names here directly map to the Rust `Setting` struct fields,
 * which use snake_case per Rust conventions. Keeping them consistent avoids
 * the need for field renaming or transformation layers.
 */
export interface Settings {
	theme: "light" | "dark" | "system";
	show_in_tray: boolean;
	launch_at_login: boolean;
	enable_logging: boolean;
	log_level: string;
	enable_notifications: boolean;
	sidebar_expanded: boolean;
	background_check_enabled: boolean;
	background_check_interval_minutes: number;
	enable_headless_browser: boolean;
	color_palette: string;
	display_timezone: string;
	date_format: string;
	preferred_currency: string;
	updated_at: string;
}

export type UpdateSettingsInput = Partial<Omit<Settings, "updated_at">>;

/**
 * Return type for the useSettings hook.
 * Provides explicit typing for better API discoverability and IDE support.
 */
export interface UseSettingsReturn {
	settings: Settings | undefined;
	isLoading: boolean;
	error: Error | null;
	updateSettings: (input: UpdateSettingsInput) => void;
	updateSettingsAsync: (input: UpdateSettingsInput) => Promise<Settings>;
	isUpdating: boolean;
}

export function useSettings(): UseSettingsReturn {
	const queryClient = useQueryClient();

	const {
		data: settings,
		isLoading,
		error,
	} = useQuery({
		queryKey: QUERY_KEYS.SETTINGS,
		queryFn: () => invoke<Settings>(COMMANDS.GET_SETTINGS),
	});

	const updateSettingsMutation = useMutation({
		mutationFn: (input: UpdateSettingsInput) =>
			invoke<Settings>(COMMANDS.UPDATE_SETTINGS, { input }),
		onSuccess: (data) => {
			queryClient.setQueryData(QUERY_KEYS.SETTINGS, data);
		},
	});

	return {
		settings,
		isLoading,
		error,
		updateSettings: updateSettingsMutation.mutate,
		updateSettingsAsync: updateSettingsMutation.mutateAsync,
		isUpdating: updateSettingsMutation.isPending,
	};
}
