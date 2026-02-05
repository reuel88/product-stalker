import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";

import { COMMANDS, EVENTS, QUERY_KEYS } from "@/constants";
import type {
	AvailabilityCheckResponse,
	BulkCheckSummary,
	CheckProgressEvent,
} from "@/modules/products/types";

/**
 * Hook for managing availability checks for a single product.
 *
 * Provides the latest availability status and a function to trigger new checks.
 * Automatically invalidates related queries when a check completes.
 *
 * @param productId - The UUID of the product to check
 * @returns Object containing:
 *   - `latestCheck`: Most recent availability check result (null if never checked)
 *   - `isLoading`: Whether the initial query is in progress
 *   - `error`: Error from the last query, if any
 *   - `checkAvailability`: Async function to trigger a new availability check
 *   - `isChecking`: Whether a check operation is currently in progress
 */
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

/**
 * Hook for fetching the availability check history for a product.
 *
 * Returns a list of past availability checks ordered by most recent first.
 * Useful for displaying price and availability trends over time.
 *
 * @param productId - The UUID of the product
 * @param limit - Optional maximum number of history records to return
 * @returns Object containing:
 *   - `history`: Array of past availability checks
 *   - `isLoading`: Whether the query is in progress
 *   - `error`: Error from the query, if any
 */
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

export interface CheckProgress {
	currentIndex: number;
	totalCount: number;
}

/**
 * Hook for triggering availability checks on all tracked products.
 *
 * Performs a bulk check with rate limiting between requests to avoid
 * overwhelming target servers. Returns a summary of results including
 * back-in-stock and price drop counts.
 *
 * Streams progress updates via Tauri events, updating the UI progressively
 * as each product is checked.
 *
 * @returns Object containing:
 *   - `checkAllAvailability`: Async function to trigger bulk check
 *   - `isCheckingAll`: Whether the bulk check is in progress
 *   - `lastSummary`: Summary from the most recent bulk check
 *   - `progress`: Current progress (currentIndex/totalCount) or null if not checking
 */
export function useCheckAllAvailability() {
	const queryClient = useQueryClient();
	const [progress, setProgress] = useState<CheckProgress | null>(null);
	const unlistenRef = useRef<UnlistenFn | null>(null);

	const setupListener = useCallback(async () => {
		// Clean up any existing listener
		if (unlistenRef.current) {
			unlistenRef.current();
			unlistenRef.current = null;
		}

		unlistenRef.current = await listen<CheckProgressEvent>(
			EVENTS.AVAILABILITY_CHECK_PROGRESS,
			(event) => {
				setProgress({
					currentIndex: event.payload.current_index,
					totalCount: event.payload.total_count,
				});

				// Optimistically update the individual product's availability cache
				const productId = event.payload.result.product_id;
				queryClient.invalidateQueries({
					queryKey: QUERY_KEYS.availability(productId),
				});
			},
		);
	}, [queryClient]);

	const cleanupListener = useCallback(() => {
		if (unlistenRef.current) {
			unlistenRef.current();
			unlistenRef.current = null;
		}
		setProgress(null);
	}, []);

	// Cleanup on unmount
	useEffect(() => {
		return () => {
			if (unlistenRef.current) {
				unlistenRef.current();
			}
		};
	}, []);

	const checkAllMutation = useMutation({
		mutationFn: async () => {
			await setupListener();
			return invoke<BulkCheckSummary>(COMMANDS.CHECK_ALL_AVAILABILITY);
		},
		onSettled: () => {
			cleanupListener();
			// Invalidate all availability queries to ensure final consistency
			queryClient.invalidateQueries({
				predicate: (query) =>
					query.queryKey[0] === QUERY_KEYS.AVAILABILITY_PREFIX,
			});
		},
	});

	return {
		checkAllAvailability: checkAllMutation.mutateAsync,
		isCheckingAll: checkAllMutation.isPending,
		lastSummary: checkAllMutation.data,
		progress,
	};
}
