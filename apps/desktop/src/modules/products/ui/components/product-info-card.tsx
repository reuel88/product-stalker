import { Loader2, RefreshCw } from "lucide-react";

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
import { useDateFormat } from "@/modules/shared/hooks/useDateFormat";
import { PriceChangeIndicator } from "./price-change-indicator";
import { STATUS_BADGE_CONFIG } from "./status-config";

interface ProductInfoCardProps {
	product: ProductResponse;
	latestCheck: AvailabilityCheckResponse | null | undefined;
	isChecking?: boolean;
	onCheck?: () => void;
}

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
	const { formatDate } = useDateFormat();
	const hasPrice = latestCheck?.price_minor_units != null;

	return (
		<Card>
			<CardHeader className="flex-row items-start justify-between gap-4">
				<div className="min-w-0 flex-1 space-y-1">
					<CardTitle className="text-lg">{product.name}</CardTitle>
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
							currentPriceMinorUnits={latestCheck.price_minor_units}
							todayAverageMinorUnits={
								latestCheck.today_average_price_minor_units
							}
							yesterdayAverageMinorUnits={
								latestCheck.yesterday_average_price_minor_units
							}
							currency={latestCheck.price_currency}
							currencyExponent={latestCheck.currency_exponent ?? 2}
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
				<span>Added: {formatDate(product.created_at)}</span>
				<span>Updated: {formatDate(product.updated_at)}</span>
			</CardFooter>
		</Card>
	);
}
