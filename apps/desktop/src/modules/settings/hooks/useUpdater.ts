import { useMutation, useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS } from "@/constants";

export type UpdateInfo =
	| { available: true; version: string; body: string | null }
	| { available: false; version: null; body: null };

export function useUpdater() {
	const { data: currentVersion, isLoading: isLoadingVersion } = useQuery({
		queryKey: ["currentVersion"],
		queryFn: () => invoke<string>(COMMANDS.GET_CURRENT_VERSION),
		staleTime: Number.POSITIVE_INFINITY,
	});

	const checkForUpdateMutation = useMutation({
		mutationFn: () => invoke<UpdateInfo>(COMMANDS.CHECK_FOR_UPDATE),
	});

	const installUpdateMutation = useMutation({
		mutationFn: () => invoke<void>(COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE),
	});

	return {
		currentVersion,
		isLoadingVersion,
		updateInfo: checkForUpdateMutation.data,
		isChecking: checkForUpdateMutation.isPending,
		checkError: checkForUpdateMutation.error,
		checkForUpdate: checkForUpdateMutation.mutate,
		checkForUpdateAsync: checkForUpdateMutation.mutateAsync,
		isInstalling: installUpdateMutation.isPending,
		installError: installUpdateMutation.error,
		installUpdate: installUpdateMutation.mutate,
		installUpdateAsync: installUpdateMutation.mutateAsync,
	};
}
