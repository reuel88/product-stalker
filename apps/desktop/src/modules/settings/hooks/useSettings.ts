import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";

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
	updated_at: string;
}

export type UpdateSettingsInput = Partial<Omit<Settings, "updated_at">>;

export function useSettings() {
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
