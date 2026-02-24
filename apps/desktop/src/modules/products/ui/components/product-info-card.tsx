import { Loader2, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardFooter,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	getDisplayPrice,
	type LowestPriceComparison,
} from "@/modules/products/price-utils";
import type {
	AvailabilityCheckResponse,
	ProductResponse,
} from "@/modules/products/types";
import { useDateFormat } from "@/modules/shared/hooks/useDateFormat";
import { PriceChangeIndicator } from "./price-change-indicator";

interface ProductInfoCardProps {
	product: ProductResponse;
	latestCheck: AvailabilityCheckResponse | null | undefined;
	isChecking?: boolean;
	onCheck?: () => void;
	lowestPriceComparison?: LowestPriceComparison | null;
}

export function ProductInfoCard({
	product,
	latestCheck,
	isChecking,
	onCheck,
	lowestPriceComparison,
}: ProductInfoCardProps) {
	const { formatDate } = useDateFormat();

	const { price, currency, exponent } = getDisplayPrice(latestCheck);
	const hasPrice = price != null;

	return (
		<Card>
			<CardHeader className="flex-row items-start justify-between gap-4">
				<div className="min-w-0 flex-1 space-y-1">
					<CardTitle className="text-lg">{product.name}</CardTitle>
				</div>
				<div className="flex items-center gap-2">
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
							currentPriceMinorUnits={price}
							todayComparisonMinorUnits={
								lowestPriceComparison?.todayLowestMinorUnits ?? null
							}
							yesterdayComparisonMinorUnits={
								lowestPriceComparison?.yesterdayLowestMinorUnits ?? null
							}
							currency={currency}
							currencyExponent={exponent}
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
