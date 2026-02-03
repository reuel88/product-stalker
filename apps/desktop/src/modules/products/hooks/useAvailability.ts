import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";
import type {
	AvailabilityCheckResponse,
	BulkCheckSummary,
} from "@/modules/products/types";

export function useAvailability(productId: string) {
	const queryClient = useQueryClient();

	const {
		data: latestCheck,
		isLoading,
		error,
	} = useQuery({
		queryKey: QUERY_KEYS.availability(productId),
		queryFn: () =>
			invoke<AvailabilityCheckResponse | null>(
				COMMANDS.GET_LATEST_AVAILABILITY,
				{
					productId,
				},
			),
		enabled: !!productId,
	});

	const checkMutation = useMutation({
		mutationFn: () =>
			invoke<AvailabilityCheckResponse>(COMMANDS.CHECK_AVAILABILITY, {
				productId,
			}),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.availability(productId),
			});
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.availabilityHistory(productId),
			});
		},
	});

	return {
		latestCheck,
		isLoading,
		error,
		checkAvailability: checkMutation.mutateAsync,
		isChecking: checkMutation.isPending,
	};
}

export function useAvailabilityHistory(productId: string, limit?: number) {
	const {
		data: history,
		isLoading,
		error,
	} = useQuery({
		queryKey: QUERY_KEYS.availabilityHistory(productId, limit),
		queryFn: () =>
			invoke<AvailabilityCheckResponse[]>(COMMANDS.GET_AVAILABILITY_HISTORY, {
				productId,
				limit,
			}),
		enabled: !!productId,
	});

	return {
		history,
		isLoading,
		error,
	};
}

export function useCheckAllAvailability() {
	const queryClient = useQueryClient();

	const checkAllMutation = useMutation({
		mutationFn: () => invoke<BulkCheckSummary>(COMMANDS.CHECK_ALL_AVAILABILITY),
		onSuccess: () => {
			// Invalidate all availability queries to refresh the UI
			queryClient.invalidateQueries({
				predicate: (query) => query.queryKey[0] === "availability",
			});
		},
	});

	return {
		checkAllAvailability: checkAllMutation.mutateAsync,
		isCheckingAll: checkAllMutation.isPending,
		lastSummary: checkAllMutation.data,
	};
}
