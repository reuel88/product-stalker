import { Loader2, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useRelativeTime } from "@/hooks/useRelativeTime";
import { cn } from "@/lib/utils";
import type { AvailabilityStatus } from "@/modules/products/types";
import { STATUS_BADGE_CONFIG } from "./status-config";

interface AvailabilityBadgeProps {
	status: AvailabilityStatus | null;
	checkedAt: string | null;
	errorMessage?: string | null;
	isChecking?: boolean;
	onCheck?: () => void;
}

export function AvailabilityBadge({
	status,
	checkedAt,
	errorMessage,
	isChecking,
	onCheck,
}: AvailabilityBadgeProps) {
	const config = status ? STATUS_BADGE_CONFIG[status] : null;
	const relativeTime = useRelativeTime(checkedAt);

	// Show error message as tooltip when status is unknown and there's an error
	const showError = status === "unknown" && errorMessage;

	return (
		<div className="flex items-center gap-2">
			{config ? (
				<div className="flex flex-col items-start gap-0.5">
					<span
						className={cn(
							"inline-flex w-full items-center overflow-hidden text-ellipsis text-nowrap rounded-full px-2 py-0.5 font-medium text-xs",
							config.className,
							showError && "cursor-help",
						)}
						title={showError ? errorMessage : undefined}
					>
						{config.label}
					</span>
					{checkedAt && (
						<span
							className={cn(
								"text-[10px] text-muted-foreground",
								showError && "cursor-help",
							)}
							title={showError ? errorMessage : undefined}
						>
							{relativeTime}
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
