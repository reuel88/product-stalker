import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, Loader2, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardFooter,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { cn } from "@/lib/utils";
import type {
	AvailabilityCheckResponse,
	AvailabilityStatus,
	ProductResponse,
} from "@/modules/products/types";
import { PriceChangeIndicator } from "./price-change-indicator";

interface ProductInfoCardProps {
	product: ProductResponse;
	latestCheck: AvailabilityCheckResponse | null | undefined;
	isChecking?: boolean;
	onCheck?: () => void;
}

const STATUS_BADGE_CONFIG: Record<
	AvailabilityStatus,
	{ label: string; className: string }
> = {
	in_stock: {
		label: "In Stock",
		className:
			"bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
	},
	out_of_stock: {
		label: "Out of Stock",
		className: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
	},
	back_order: {
		label: "Back Order",
		className:
			"bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
	},
	unknown: {
		label: "Unknown",
		className: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
	},
};

function StatusBadge({ status }: { status: AvailabilityStatus }) {
	const { label, className } = STATUS_BADGE_CONFIG[status];

	return (
		<span
			className={cn(
				"inline-flex items-center rounded-full px-2.5 py-0.5 font-medium text-xs",
				className,
			)}
		>
			{label}
		</span>
	);
}

export function ProductInfoCard({
	product,
	latestCheck,
	isChecking,
	onCheck,
}: ProductInfoCardProps) {
	const hasPrice = latestCheck?.price_cents != null;

	return (
		<Card>
			<CardHeader className="flex-row items-start justify-between gap-4">
				<div className="min-w-0 flex-1 space-y-1">
					<CardTitle className="text-lg">{product.name}</CardTitle>
					<button
						type="button"
						onClick={() => openUrl(product.url)}
						className="inline-flex items-center gap-1 text-left text-primary text-xs hover:underline"
					>
						<span className="truncate">{product.url}</span>
						<ExternalLink className="size-3 shrink-0" />
					</button>
				</div>
				<div className="flex items-center gap-2">
					{latestCheck?.status && <StatusBadge status={latestCheck.status} />}
					{onCheck && (
						<Button
							variant="ghost"
							size="icon-sm"
							onClick={onCheck}
							disabled={isChecking}
							title="Check availability"
						>
							{isChecking ? (
								<Loader2 className="size-4 animate-spin" />
							) : (
								<RefreshCw className="size-4" />
							)}
						</Button>
					)}
				</div>
			</CardHeader>

			<CardContent className="space-y-4">
				{hasPrice && (
					<div>
						<p className="text-muted-foreground text-xs">Current Price</p>
						<PriceChangeIndicator
							currentPriceCents={latestCheck.price_cents}
							previousPriceCents={latestCheck.previous_price_cents}
							currency={latestCheck.price_currency}
							variant="detailed"
						/>
					</div>
				)}

				{product.description && (
					<div>
						<p className="text-muted-foreground text-xs">Description</p>
						<p className="text-sm">{product.description}</p>
					</div>
				)}

				{product.notes && (
					<div>
						<p className="text-muted-foreground text-xs">Notes</p>
						<p className="text-sm">{product.notes}</p>
					</div>
				)}
			</CardContent>

			<CardFooter className="justify-between text-muted-foreground text-xs">
				<span>Added: {new Date(product.created_at).toLocaleDateString()}</span>
				<span>
					Updated: {new Date(product.updated_at).toLocaleDateString()}
				</span>
			</CardFooter>
		</Card>
	);
}
