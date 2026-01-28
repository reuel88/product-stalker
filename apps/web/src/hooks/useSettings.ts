import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export interface Settings {
	theme: "light" | "dark" | "system";
	show_in_tray: boolean;
	launch_at_login: boolean;
	enable_logging: boolean;
	log_level: string;
	enable_notifications: boolean;
	sidebar_expanded: boolean;
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
		queryKey: ["settings"],
		queryFn: () => invoke<Settings>("get_settings"),
	});

	const updateSettingsMutation = useMutation({
		mutationFn: (input: UpdateSettingsInput) =>
			invoke<Settings>("update_settings", { input }),
		onSuccess: (data) => {
			queryClient.setQueryData(["settings"], data);
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
