import { Loader2, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import { MESSAGES } from "@/constants";
import type { AvailabilityStatus } from "@/modules/products/types";

interface AvailabilityBadgeProps {
	status: AvailabilityStatus | null;
	checkedAt: string | null;
	isChecking?: boolean;
	onCheck?: () => void;
}

const statusConfig: Record<
	AvailabilityStatus,
	{ label: string; className: string }
> = {
	in_stock: {
		label: MESSAGES.AVAILABILITY.IN_STOCK,
		className:
			"bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
	},
	out_of_stock: {
		label: MESSAGES.AVAILABILITY.OUT_OF_STOCK,
		className: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
	},
	back_order: {
		label: MESSAGES.AVAILABILITY.BACK_ORDER,
		className:
			"bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
	},
	unknown: {
		label: MESSAGES.AVAILABILITY.UNKNOWN,
		className: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
	},
};

export function AvailabilityBadge({
	status,
	checkedAt,
	isChecking,
	onCheck,
}: AvailabilityBadgeProps) {
	const config = status ? statusConfig[status] : null;

	const formatCheckedAt = (dateStr: string) => {
		const date = new Date(dateStr);
		if (Number.isNaN(date.getTime())) {
			return "Unknown";
		}
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / 60000);
		const diffHours = Math.floor(diffMs / 3600000);
		const diffDays = Math.floor(diffMs / 86400000);

		if (diffMins < 1) return "Just now";
		if (diffMins < 60) return `${diffMins}m ago`;
		if (diffHours < 24) return `${diffHours}h ago`;
		return `${diffDays}d ago`;
	};

	return (
		<div className="flex items-center gap-2">
			{config ? (
				<div className="flex flex-col items-start gap-0.5">
					<span
						className={`inline-flex items-center rounded-full px-2 py-0.5 font-medium text-xs ${config.className}`}
					>
						{config.label}
					</span>
					{checkedAt && (
						<span className="text-[10px] text-muted-foreground">
							{formatCheckedAt(checkedAt)}
						</span>
					)}
				</div>
			) : (
				<span className="text-muted-foreground text-xs">Not checked</span>
			)}
			{onCheck && (
				<Button
					variant="ghost"
					size="icon-sm"
					onClick={onCheck}
					disabled={isChecking}
					title="Check availability"
				>
					{isChecking ? (
						<Loader2 className="size-3.5 animate-spin" />
					) : (
						<RefreshCw className="size-3.5" />
					)}
				</Button>
			)}
		</div>
	);
}
