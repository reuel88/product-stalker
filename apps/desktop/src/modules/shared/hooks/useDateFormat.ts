import { useMemo } from "react";
import {
	formatChartAxisDate as formatChartAxisDateUtil,
	formatChartTooltipDate as formatChartTooltipDateUtil,
	formatDateTime as formatDateTimeUtil,
	formatDate as formatDateUtil,
	getDateRangeLabel as getDateRangeLabelUtil,
} from "@/lib/format-date";
import { useSettings } from "@/modules/settings/hooks/useSettings";

/**
 * Hook that provides memoized date formatting functions based on user settings
 *
 * Reads the display_timezone and date_format settings and returns formatters
 * that automatically apply these preferences.
 *
 * @returns Object containing:
 *   - formatDate: Format date-only strings
 *   - formatDateTime: Format date and time strings
 *   - formatChartAxisDate: Format dates for chart axes (short)
 *   - formatChartTooltipDate: Format dates for chart tooltips (full)
 *   - getDateRangeLabel: Format date ranges
 *   - dateFormat: Current date format setting
 *   - timezone: Current timezone setting
 */
export function useDateFormat() {
	const { settings } = useSettings();

	const dateFormat = settings?.date_format ?? "system";
	const timezone = settings?.display_timezone ?? "auto";

	const formatters = useMemo(
		() => ({
			formatDate: (dateString: string) =>
				formatDateUtil(dateString, dateFormat as never, timezone),
			formatDateTime: (dateString: string) =>
				formatDateTimeUtil(dateString, dateFormat as never, timezone),
			formatChartAxisDate: (dateString: string) =>
				formatChartAxisDateUtil(dateString, timezone),
			formatChartTooltipDate: (dateString: string) =>
				formatChartTooltipDateUtil(dateString, dateFormat as never, timezone),
			getDateRangeLabel: (dates: string[]) =>
				getDateRangeLabelUtil(dates, dateFormat as never, timezone),
		}),
		[dateFormat, timezone],
	);

	return {
		...formatters,
		dateFormat,
		timezone,
	};
}
