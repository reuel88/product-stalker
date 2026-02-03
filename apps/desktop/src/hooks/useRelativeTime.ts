import { useEffect, useState } from "react";

function formatRelativeTime(dateStr: string): string {
	const date = new Date(dateStr);
	if (Number.isNaN(date.getTime())) return "Unknown";

	const now = new Date();
	const diffMs = now.getTime() - date.getTime();
	const diffMins = Math.floor(diffMs / 60000);
	const diffHours = Math.floor(diffMs / 3600000);
	const diffDays = Math.floor(diffMs / 86400000);

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
		if (!dateStr) return;

		// Update immediately
		setRelativeTime(formatRelativeTime(dateStr));

		// Update every minute
		const interval = setInterval(() => {
			setRelativeTime(formatRelativeTime(dateStr));
		}, 60000);

		return () => clearInterval(interval);
	}, [dateStr]);

	return relativeTime;
}
