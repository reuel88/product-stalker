import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import { COMMANDS, QUERY_KEYS } from "@/constants";

export interface ExchangeRate {
	id: number;
	from_currency: string;
	to_currency: string;
	rate: number;
	source: string;
	fetched_at: string;
}

export interface UseExchangeRatesReturn {
	rates: ExchangeRate[] | undefined;
	isLoading: boolean;
	error: Error | null;
	refreshRates: () => void;
	refreshRatesAsync: () => Promise<void>;
	isRefreshing: boolean;
	setManualRate: (params: { from: string; to: string; rate: number }) => void;
	deleteRate: (params: { from: string; to: string }) => void;
}

export function useExchangeRates(): UseExchangeRatesReturn {
	const queryClient = useQueryClient();

	const {
		data: rates,
		isLoading,
		error,
	} = useQuery({
		queryKey: QUERY_KEYS.EXCHANGE_RATES,
		queryFn: () => invoke<ExchangeRate[]>(COMMANDS.GET_EXCHANGE_RATES),
	});

	const refreshMutation = useMutation({
		mutationFn: () => invoke<void>(COMMANDS.REFRESH_EXCHANGE_RATES),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.EXCHANGE_RATES,
			});
		},
	});

	const setManualMutation = useMutation({
		mutationFn: (params: { from: string; to: string; rate: number }) =>
			invoke<void>(COMMANDS.SET_MANUAL_EXCHANGE_RATE, params),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.EXCHANGE_RATES,
			});
		},
	});

	const deleteMutation = useMutation({
		mutationFn: (params: { from: string; to: string }) =>
			invoke<void>(COMMANDS.DELETE_EXCHANGE_RATE, params),
		onSuccess: () => {
			queryClient.invalidateQueries({
				queryKey: QUERY_KEYS.EXCHANGE_RATES,
			});
		},
	});

	return {
		rates,
		isLoading,
		error,
		refreshRates: refreshMutation.mutate,
		refreshRatesAsync: refreshMutation.mutateAsync,
		isRefreshing: refreshMutation.isPending,
		setManualRate: setManualMutation.mutate,
		deleteRate: deleteMutation.mutate,
	};
}
