import {
	flexRender,
	getCoreRowModel,
	getPaginationRowModel,
	useReactTable,
} from "@tanstack/react-table";
import {
	ChevronFirst,
	ChevronLast,
	ChevronLeft,
	ChevronRight,
} from "lucide-react";
import type { ReactNode } from "react";
import { createContext, useContext, useMemo } from "react";

import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { UI } from "@/constants";
import { cn } from "@/lib/utils";
import { useAvailability } from "@/modules/products/hooks/useAvailability";
import type {
	AvailabilityCheckResponse,
	ProductResponse,
} from "@/modules/products/types";
import { useDateFormat } from "@/modules/shared/hooks/useDateFormat";
import { AvailabilityBadge } from "./availability-badge";
import { PriceChangeIndicator } from "./price-change-indicator";
import { createProductColumns } from "./products-table-columns";

interface ProductsTableProps {
	products: ProductResponse[];
	isLoading?: boolean;
	onEdit?: (product: ProductResponse) => void;
	onDelete?: (product: ProductResponse) => void;
}

interface ProductAvailabilityData {
	latestCheck: AvailabilityCheckResponse | null | undefined;
	isChecking: boolean;
	handleCheck: () => Promise<void>;
}

const ProductAvailabilityContext =
	createContext<ProductAvailabilityData | null>(null);

function useProductAvailabilityData() {
	const context = useContext(ProductAvailabilityContext);
	if (!context) {
		throw new Error(
			"useProductAvailabilityData must be used within a ProductAvailabilityProvider",
		);
	}
	return context;
}

/**
 * Row-level provider that shares a single useAvailability subscription
 * between AvailabilityCell and PriceCell, avoiding duplicate queries
 * and ensuring shared mutation state (e.g. isChecking).
 */
function ProductAvailabilityProvider({
	productId,
	children,
}: {
	productId: string;
	children: ReactNode;
}) {
	const { latestCheck, isChecking, checkWithToast } =
		useAvailability(productId);

	const value = useMemo(
		() => ({ latestCheck, isChecking, handleCheck: checkWithToast }),
		[latestCheck, isChecking, checkWithToast],
	);

	return (
		<ProductAvailabilityContext.Provider value={value}>
			{children}
		</ProductAvailabilityContext.Provider>
	);
}

function AvailabilityCell() {
	const { latestCheck, isChecking, handleCheck } = useProductAvailabilityData();

	return (
		<AvailabilityBadge
			status={latestCheck?.status ?? null}
			checkedAt={latestCheck?.checked_at ?? null}
			errorMessage={latestCheck?.error_message}
			isChecking={isChecking}
			onCheck={handleCheck}
		/>
	);
}

function PriceCell({ productId }: { productId: string }) {
	const { latestCheck } = useProductAvailabilityData();

	return (
		<span data-testid={`price-${productId}`}>
			<PriceChangeIndicator
				currentPriceMinorUnits={latestCheck?.price_minor_units ?? null}
				todayAverageMinorUnits={
					latestCheck?.today_average_price_minor_units ?? null
				}
				yesterdayAverageMinorUnits={
					latestCheck?.yesterday_average_price_minor_units ?? null
				}
				currency={latestCheck?.price_currency ?? null}
				currencyExponent={latestCheck?.currency_exponent ?? 2}
				variant="compact"
			/>
		</span>
	);
}

export function ProductsTable({
	products,
	isLoading,
	onEdit,
	onDelete,
}: ProductsTableProps) {
	const { formatDate } = useDateFormat();

	const columns = useMemo(
		() =>
			createProductColumns({
				onEdit,
				onDelete,
				AvailabilityCell,
				PriceCell,
				formatDate,
			}),
		[onEdit, onDelete, formatDate],
	);

	const table = useReactTable({
		data: products,
		columns,
		getCoreRowModel: getCoreRowModel(),
		getPaginationRowModel: getPaginationRowModel(),
		initialState: {
			pagination: {
				pageSize: UI.PAGINATION.DEFAULT_PAGE_SIZE,
			},
		},
	});

	if (isLoading) {
		return <ProductsTableSkeleton />;
	}

	return (
		<div className="space-y-4">
			<Table>
				<TableHeader>
					{table.getHeaderGroups().map((headerGroup) => (
						<TableRow key={headerGroup.id}>
							{headerGroup.headers.map((header) => (
								<TableHead key={header.id}>
									{header.isPlaceholder
										? null
										: flexRender(
												header.column.columnDef.header,
												header.getContext(),
											)}
								</TableHead>
							))}
						</TableRow>
					))}
				</TableHeader>
				<TableBody>
					{table.getRowModel().rows?.length ? (
						table.getRowModel().rows.map((row) => (
							<ProductAvailabilityProvider
								key={row.id}
								productId={row.original.id}
							>
								<TableRow data-state={row.getIsSelected() && "selected"}>
									{row.getVisibleCells().map((cell) => (
										<TableCell key={cell.id}>
											{flexRender(
												cell.column.columnDef.cell,
												cell.getContext(),
											)}
										</TableCell>
									))}
								</TableRow>
							</ProductAvailabilityProvider>
						))
					) : (
						<TableRow>
							<TableCell colSpan={columns.length} className="h-24 text-center">
								No products found
							</TableCell>
						</TableRow>
					)}
				</TableBody>
			</Table>

			<div className="flex items-center justify-between">
				<div className="text-muted-foreground text-xs">
					Page {table.getState().pagination.pageIndex + 1} of{" "}
					{table.getPageCount() || 1}
				</div>
				<div className="flex items-center gap-1">
					<Button
						variant="outline"
						size="icon-sm"
						onClick={() => table.firstPage()}
						disabled={!table.getCanPreviousPage()}
					>
						<ChevronFirst className="size-4" />
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						onClick={() => table.previousPage()}
						disabled={!table.getCanPreviousPage()}
					>
						<ChevronLeft className="size-4" />
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						onClick={() => table.nextPage()}
						disabled={!table.getCanNextPage()}
					>
						<ChevronRight className="size-4" />
					</Button>
					<Button
						variant="outline"
						size="icon-sm"
						onClick={() => table.lastPage()}
						disabled={!table.getCanNextPage()}
					>
						<ChevronLast className="size-4" />
					</Button>
				</div>
			</div>
		</div>
	);
}

/** Skeleton widths that approximate the content they represent */
const SKELETON_WIDTHS = {
	name: "w-24",
	url: "w-40",
	availabilityBadge: "h-6 w-20",
	price: "w-16",
	description: "w-32",
	date: "w-20",
	actionButton: "size-7",
	paginationText: "w-24",
} as const;

function ProductsTableSkeleton() {
	return (
		<div className="space-y-4">
			<Table>
				<TableHeader>
					<TableRow>
						<TableHead>Name</TableHead>
						<TableHead>URL</TableHead>
						<TableHead>Availability</TableHead>
						<TableHead>Price</TableHead>
						<TableHead>Description</TableHead>
						<TableHead>Created</TableHead>
						<TableHead />
					</TableRow>
				</TableHeader>
				<TableBody>
					{Array.from({ length: UI.SKELETON.TABLE_ROW_COUNT }).map((_, i) => (
						// biome-ignore lint/suspicious/noArrayIndexKey: Static skeleton rows never reorder
						<TableRow key={i}>
							<TableCell>
								<Skeleton className={cn("h-4", SKELETON_WIDTHS.name)} />
							</TableCell>
							<TableCell>
								<Skeleton className={cn("h-4", SKELETON_WIDTHS.url)} />
							</TableCell>
							<TableCell>
								<Skeleton className={SKELETON_WIDTHS.availabilityBadge} />
							</TableCell>
							<TableCell>
								<Skeleton className={cn("h-4", SKELETON_WIDTHS.price)} />
							</TableCell>
							<TableCell>
								<Skeleton className={cn("h-4", SKELETON_WIDTHS.description)} />
							</TableCell>
							<TableCell>
								<Skeleton className={cn("h-4", SKELETON_WIDTHS.date)} />
							</TableCell>
							<TableCell>
								<Skeleton className={SKELETON_WIDTHS.actionButton} />
							</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>
			<div className="flex items-center justify-between">
				<Skeleton className={cn("h-4", SKELETON_WIDTHS.paginationText)} />
				<div className="flex items-center gap-1">
					<Skeleton className={SKELETON_WIDTHS.actionButton} />
					<Skeleton className={SKELETON_WIDTHS.actionButton} />
					<Skeleton className={SKELETON_WIDTHS.actionButton} />
					<Skeleton className={SKELETON_WIDTHS.actionButton} />
				</div>
			</div>
		</div>
	);
}
