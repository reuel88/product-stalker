import { Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { useState } from "react";

import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import {
	useAvailability,
	useAvailabilityHistory,
} from "@/modules/products/hooks/useAvailability";
import { useProduct } from "@/modules/products/hooks/useProduct";
import {
	filterByTimeRange,
	transformToPriceDataPoints,
} from "@/modules/products/price-utils";
import type { TimeRange } from "@/modules/products/types";
import { PriceHistoryChart } from "@/modules/products/ui/components/price-history-chart";
import { ProductInfoCard } from "@/modules/products/ui/components/product-info-card";
import { TimeRangeSelector } from "@/modules/products/ui/components/time-range-selector";

interface ProductDetailViewProps {
	productId: string;
}

function LoadingSkeleton() {
	return (
		<div className="space-y-6">
			<Skeleton className="h-8 w-48" />
			<Skeleton className="h-50 w-full" />
			<Skeleton className="h-75 w-full" />
		</div>
	);
}

function ErrorState({ message }: { message: string }) {
	return (
		<div className="flex flex-col items-center justify-center gap-4 py-12">
			<p className="text-destructive text-sm">{message}</p>
			<Link
				to="/products"
				className="inline-flex items-center gap-1.5 rounded-none border border-border bg-background px-2.5 py-1.5 font-medium text-xs hover:bg-muted"
			>
				<ArrowLeft className="size-4" />
				Back to Products
			</Link>
		</div>
	);
}

export function ProductDetailView({ productId }: ProductDetailViewProps) {
	const [timeRange, setTimeRange] = useState<TimeRange>("30d");

	const {
		product,
		isLoading: isLoadingProduct,
		error: productError,
	} = useProduct(productId);
	const { latestCheck, checkWithToast, isChecking } =
		useAvailability(productId);
	const { history, isLoading: isLoadingHistory } =
		useAvailabilityHistory(productId);

	if (isLoadingProduct) {
		return <LoadingSkeleton />;
	}

	if (productError || !product) {
		return (
			<ErrorState message={productError?.message ?? "Product not found"} />
		);
	}

	const filteredHistory = history ? filterByTimeRange(history, timeRange) : [];
	const priceDataPoints = transformToPriceDataPoints(filteredHistory);

	return (
		<div className="container mx-auto space-y-6 overflow-y-auto px-4 py-6">
			<Link
				to="/products"
				className="inline-flex items-center gap-1 rounded-none px-2.5 py-1.5 font-medium text-xs hover:bg-muted"
			>
				<ArrowLeft className="size-4" />
				Back to Products
			</Link>

			<ProductInfoCard
				product={product}
				latestCheck={latestCheck}
				isChecking={isChecking}
				onCheck={checkWithToast}
			/>

			<Card>
				<CardHeader className="flex-row items-center justify-between">
					<div>
						<CardTitle>Price History</CardTitle>
						<CardDescription>Track how the price has changed</CardDescription>
					</div>
					<TimeRangeSelector value={timeRange} onChange={setTimeRange} />
				</CardHeader>
				<CardContent>
					{isLoadingHistory ? (
						<Skeleton className="h-50 w-full" />
					) : (
						<PriceHistoryChart data={priceDataPoints} />
					)}
				</CardContent>
			</Card>
		</div>
	);
}
