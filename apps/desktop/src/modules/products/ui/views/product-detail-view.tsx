import { Link } from "@tanstack/react-router";
import { ArrowLeft, Plus } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { MESSAGES } from "@/constants";
import { withToast, withToastVoid } from "@/lib/toast-helpers";
import {
	useAvailability,
	useAvailabilityHistory,
} from "@/modules/products/hooks/useAvailability";
import { useProduct } from "@/modules/products/hooks/useProduct";
import { useProductRetailers } from "@/modules/products/hooks/useProductRetailers";
import {
	filterByTimeRange,
	transformToPriceDataPoints,
} from "@/modules/products/price-utils";
import type { TimeRange } from "@/modules/products/types";
import { AddRetailerDialog } from "@/modules/products/ui/components/add-retailer-dialog";
import { PriceHistoryChart } from "@/modules/products/ui/components/price-history-chart";
import { ProductInfoCard } from "@/modules/products/ui/components/product-info-card";
import { RetailerList } from "@/modules/products/ui/components/retailer-list";
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
	const [showAddRetailer, setShowAddRetailer] = useState(false);

	const {
		product,
		isLoading: isLoadingProduct,
		error: productError,
	} = useProduct(productId);
	const { latestCheck, checkWithToast, isChecking } =
		useAvailability(productId);
	const { history, isLoading: isLoadingHistory } =
		useAvailabilityHistory(productId);
	const { retailers, addRetailer, isAdding, removeRetailer, isRemoving } =
		useProductRetailers(productId);

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

	const handleAddRetailer = async (url: string, label: string | null) => {
		if (!url) {
			toast.error(MESSAGES.VALIDATION.URL_REQUIRED);
			return;
		}
		const result = await withToast(
			() => addRetailer({ product_id: productId, url, label }),
			{
				success: MESSAGES.RETAILER.ADDED,
				error: MESSAGES.RETAILER.ADD_FAILED,
			},
		);
		if (result) setShowAddRetailer(false);
	};

	const handleRemoveRetailer = async (id: string) => {
		await withToastVoid(() => removeRetailer(id), {
			success: MESSAGES.RETAILER.REMOVED,
			error: MESSAGES.RETAILER.REMOVE_FAILED,
		});
	};

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
						<CardTitle>Retailers</CardTitle>
						<CardDescription>URLs tracked for this product</CardDescription>
					</div>
					<Button size="sm" onClick={() => setShowAddRetailer(true)}>
						<Plus className="size-4" />
						Add Retailer
					</Button>
				</CardHeader>
				<CardContent>
					<RetailerList
						retailers={retailers ?? []}
						onRemove={handleRemoveRetailer}
						isRemoving={isRemoving}
					/>
				</CardContent>
			</Card>

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

			<AddRetailerDialog
				open={showAddRetailer}
				onOpenChange={setShowAddRetailer}
				onSubmit={handleAddRetailer}
				isSubmitting={isAdding}
			/>
		</div>
	);
}
