import {
	closestCenter,
	DndContext,
	type DragEndEvent,
	KeyboardSensor,
	PointerSensor,
	useSensor,
	useSensors,
} from "@dnd-kit/core";
import {
	arrayMove,
	SortableContext,
	useSortable,
	verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
	ExternalLink,
	GripVertical,
	Store,
	Trash2,
	TrendingDown,
	TrendingUp,
} from "lucide-react";
import type { CSSProperties } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
	calculatePriceChangePercent,
	extractDomain,
	formatPrice,
	formatPriceChangePercent,
	getPriceChangeDirection,
	isRoundedZero,
	type RetailerDetails,
} from "@/modules/products/price-utils";
import type {
	AvailabilityStatus,
	ProductRetailerResponse,
} from "@/modules/products/types";

const STATUS_DOT_CONFIG: Record<
	AvailabilityStatus,
	{ color: string; label: string }
> = {
	in_stock: { color: "bg-green-500", label: "In Stock" },
	out_of_stock: { color: "bg-red-500", label: "Out of Stock" },
	back_order: { color: "bg-yellow-500", label: "Back Order" },
	unknown: { color: "bg-gray-400", label: "Unknown" },
};

function StatusDot({ status }: { status: AvailabilityStatus }) {
	return (
		<span
			className={cn(
				"size-2 shrink-0 rounded-full",
				STATUS_DOT_CONFIG[status].color,
			)}
			title={STATUS_DOT_CONFIG[status].label}
		/>
	);
}

/**
 * Lightweight price change badge showing only direction icon + percent.
 * Similar to PriceChangeIndicator's compact variant but without price text,
 * used alongside an already-rendered price in the retailer list.
 */
function PriceChangeBadge({
	todayComparisonMinorUnits,
	yesterdayComparisonMinorUnits,
}: {
	todayComparisonMinorUnits: number | null;
	yesterdayComparisonMinorUnits: number | null;
}) {
	const direction = getPriceChangeDirection(
		todayComparisonMinorUnits,
		yesterdayComparisonMinorUnits,
	);
	if (direction === "unchanged" || direction === "unknown") return null;

	const percent = calculatePriceChangePercent(
		todayComparisonMinorUnits,
		yesterdayComparisonMinorUnits,
	);
	const roundedZero = isRoundedZero(
		todayComparisonMinorUnits,
		yesterdayComparisonMinorUnits,
	);
	const Icon = direction === "down" ? TrendingDown : TrendingUp;
	const colorClass =
		direction === "down"
			? "text-green-600 dark:text-green-400"
			: "text-red-600 dark:text-red-400";

	return (
		<span
			className={cn(
				"inline-flex items-center gap-0.5 font-medium text-xs",
				colorClass,
			)}
		>
			<Icon className="size-3" />
			{!roundedZero && formatPriceChangePercent(percent)}
		</span>
	);
}

interface RetailerListProps {
	retailers: ProductRetailerResponse[];
	onRemove: (id: string) => void;
	isRemoving: boolean;
	retailerDetails?: Map<string, RetailerDetails>;
	cheapestRetailerId?: string | null;
	onReorder?: (newOrder: ProductRetailerResponse[]) => void;
}

function SortableRetailerItem({
	retailer,
	onRemove,
	isRemoving,
	details,
	isCheapest,
}: {
	retailer: ProductRetailerResponse;
	onRemove: (id: string) => void;
	isRemoving: boolean;
	details?: RetailerDetails;
	isCheapest: boolean;
}) {
	const { attributes, listeners, setNodeRef, transform, transition } =
		useSortable({ id: retailer.id });

	const style: CSSProperties = {
		transform: CSS.Transform.toString(transform),
		transition,
	};

	return (
		<div
			ref={setNodeRef}
			style={style}
			className="flex items-center justify-between gap-4 py-3"
		>
			<button
				type="button"
				className="cursor-grab touch-none text-muted-foreground hover:text-foreground"
				aria-label="Drag to reorder"
				data-testid={`drag-handle-${retailer.id}`}
				{...attributes}
				{...listeners}
			>
				<GripVertical className="size-4" />
			</button>
			<div className="min-w-0 flex-1">
				<div className="flex items-center gap-2">
					<Store className="size-4 shrink-0 text-muted-foreground" />
					{details?.status && <StatusDot status={details.status} />}
					<span className="truncate font-medium text-sm">
						{extractDomain(retailer.url)}
					</span>
					{retailer.label && (
						<span className="shrink-0 rounded bg-muted px-1.5 py-0.5 text-muted-foreground text-xs">
							{retailer.label}
						</span>
					)}
				</div>
				<button
					type="button"
					onClick={() => openUrl(retailer.url)}
					className="mt-0.5 flex max-w-full items-center gap-1 text-left text-primary text-xs hover:underline"
				>
					<span className="truncate">{retailer.url}</span>
					<ExternalLink className="size-3 shrink-0" />
				</button>
			</div>
			{details?.priceMinorUnits != null && (
				<div className="shrink-0 text-right">
					<span
						className={cn(
							"font-semibold text-sm",
							isCheapest
								? "text-green-600 dark:text-green-400"
								: "text-foreground",
						)}
					>
						{formatPrice(
							details.priceMinorUnits,
							details.currency,
							details.currencyExponent,
						)}
					</span>
					{details.originalCurrency && (
						<span className="block text-muted-foreground text-xs">
							{formatPrice(
								details.originalPriceMinorUnits ?? null,
								details.originalCurrency,
								details.originalCurrencyExponent,
							)}
						</span>
					)}
					<PriceChangeBadge
						todayComparisonMinorUnits={details.todayAverageMinorUnits}
						yesterdayComparisonMinorUnits={details.yesterdayAverageMinorUnits}
					/>
				</div>
			)}
			<Button
				variant="ghost"
				size="icon-sm"
				onClick={() => onRemove(retailer.id)}
				disabled={isRemoving}
				title="Remove retailer"
			>
				<Trash2 className="size-4" />
			</Button>
		</div>
	);
}

export function RetailerList({
	retailers,
	onRemove,
	isRemoving,
	retailerDetails,
	cheapestRetailerId,
	onReorder,
}: RetailerListProps) {
	const sensors = useSensors(
		useSensor(PointerSensor),
		useSensor(KeyboardSensor),
	);

	if (retailers.length === 0) {
		return (
			<p className="py-4 text-center text-muted-foreground text-sm">
				No retailers added yet. Add a retailer URL to start tracking.
			</p>
		);
	}

	const handleDragEnd = (event: DragEndEvent) => {
		const { active, over } = event;
		if (!over || active.id === over.id || !onReorder) return;

		const oldIndex = retailers.findIndex((r) => r.id === active.id);
		const newIndex = retailers.findIndex((r) => r.id === over.id);
		const newOrder = arrayMove(retailers, oldIndex, newIndex);
		onReorder(newOrder);
	};

	return (
		<DndContext
			sensors={sensors}
			collisionDetection={closestCenter}
			onDragEnd={handleDragEnd}
		>
			<SortableContext
				items={retailers.map((r) => r.id)}
				strategy={verticalListSortingStrategy}
			>
				<div className="divide-y">
					{retailers.map((retailer) => {
						const details = retailerDetails?.get(retailer.id);
						return (
							<SortableRetailerItem
								key={retailer.id}
								retailer={retailer}
								onRemove={onRemove}
								isRemoving={isRemoving}
								details={details}
								isCheapest={cheapestRetailerId === retailer.id}
							/>
						);
					})}
				</div>
			</SortableContext>
		</DndContext>
	);
}
