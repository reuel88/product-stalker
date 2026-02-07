import { useEffect, useState } from "react";

const MS_PER_MINUTE = 60_000;
const MS_PER_HOUR = 3_600_000;
const MS_PER_DAY = 86_400_000;

function formatRelativeTime(dateStr: string): string {
	const date = new Date(dateStr);
	if (Number.isNaN(date.getTime())) return "Unknown";

	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffMins = Math.floor(diffMs / MS_PER_MINUTE);
	const diffHours = Math.floor(diffMs / MS_PER_HOUR);
	const diffDays = Math.floor(diffMs / MS_PER_DAY);

	if (diffMins < 1) return "Just now";
	if (diffMins < 60) return `${diffMins}m ago`;
	if (diffHours < 24) return `${diffHours}h ago`;
	return `${diffDays}d ago`;
}

export function useRelativeTime(dateStr: string | null): string {
	const [relativeTime, setRelativeTime] = useState(() =>
		dateStr ? formatRelativeTime(dateStr) : "",
	);

	useEffect(() => {
		if (!dateStr) {
			setRelativeTime("");
			return;
		}

		// Update immediately
		setRelativeTime(formatRelativeTime(dateStr));

		// Update every minute
		const interval = setInterval(() => {
			setRelativeTime(formatRelativeTime(dateStr));
		}, MS_PER_MINUTE);

		return () => clearInterval(interval);
	}, [dateStr]);

	return relativeTime;
}
