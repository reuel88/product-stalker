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
import { ExternalLink, GripVertical, Store, Trash2 } from "lucide-react";
import type { CSSProperties } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import {
	extractDomain,
	formatPrice,
	type RetailerPrice,
} from "@/modules/products/price-utils";
import type { ProductRetailerResponse } from "@/modules/products/types";

interface RetailerListProps {
	retailers: ProductRetailerResponse[];
	onRemove: (id: string) => void;
	isRemoving: boolean;
	retailerPrices?: Map<string, RetailerPrice>;
	cheapestRetailerId?: string | null;
	onReorder?: (newOrder: ProductRetailerResponse[]) => void;
}

function SortableRetailerItem({
	retailer,
	onRemove,
	isRemoving,
	price,
	isCheapest,
}: {
	retailer: ProductRetailerResponse;
	onRemove: (id: string) => void;
	isRemoving: boolean;
	price?: RetailerPrice;
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
					className="mt-0.5 inline-flex items-center gap-1 text-left text-primary text-xs hover:underline"
				>
					<span className="truncate">{retailer.url}</span>
					<ExternalLink className="size-3 shrink-0" />
				</button>
			</div>
			{price && (
				<span
					className={cn(
						"shrink-0 font-semibold text-sm",
						isCheapest
							? "text-green-600 dark:text-green-400"
							: "text-foreground",
					)}
				>
					{formatPrice(
						price.priceMinorUnits,
						price.currency,
						price.currencyExponent,
					)}
				</span>
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
	retailerPrices,
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
						const price = retailerPrices?.get(retailer.id);
						return (
							<SortableRetailerItem
								key={retailer.id}
								retailer={retailer}
								onRemove={onRemove}
								isRemoving={isRemoving}
								price={price}
								isCheapest={cheapestRetailerId === retailer.id}
							/>
						);
					})}
				</div>
			</SortableContext>
		</DndContext>
	);
}
