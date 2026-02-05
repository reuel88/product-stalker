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
import { useMemo } from "react";
import { toast } from "sonner";

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
import { MESSAGES, UI } from "@/constants";
import { formatPrice } from "@/lib/utils";
import { useAvailability } from "@/modules/products/hooks/useAvailability";
import type { ProductResponse } from "@/modules/products/types";
import { AvailabilityBadge } from "./availability-badge";
import { createProductColumns } from "./products-table-columns";

interface ProductsTableProps {
	products: ProductResponse[];
	isLoading?: boolean;
	onEdit?: (product: ProductResponse) => void;
	onDelete?: (product: ProductResponse) => void;
}

function AvailabilityCell({ productId }: { productId: string }) {
	const { latestCheck, isChecking, checkAvailability } =
		useAvailability(productId);

	const handleCheck = async () => {
		try {
			const result = await checkAvailability();
			if (result.error_message) {
				toast.error(result.error_message);
			} else {
				toast.success(MESSAGES.AVAILABILITY.CHECKED);
			}
		} catch {
			toast.error(MESSAGES.AVAILABILITY.CHECK_FAILED);
		}
	};

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
	const { latestCheck } = useAvailability(productId);

	const price = formatPrice(
		latestCheck?.price_cents ?? null,
		latestCheck?.price_currency ?? null,
	);

	return (
		<span data-testid={`price-${productId}`} className="text-muted-foreground">
			{price}
		</span>
	);
}

export function ProductsTable({
	products,
	isLoading,
	onEdit,
	onDelete,
}: ProductsTableProps) {
	const columns = useMemo(
		() =>
			createProductColumns({
				onEdit,
				onDelete,
				AvailabilityCell,
				PriceCell,
			}),
		[onEdit, onDelete],
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
							<TableRow
								key={row.id}
								data-state={row.getIsSelected() && "selected"}
							>
								{row.getVisibleCells().map((cell) => (
									<TableCell key={cell.id}>
										{flexRender(cell.column.columnDef.cell, cell.getContext())}
									</TableCell>
								))}
							</TableRow>
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
					{Array.from({ length: 5 }).map((_, i) => (
						// biome-ignore lint/suspicious/noArrayIndexKey: Static skeleton rows never reorder
						<TableRow key={i}>
							<TableCell>
								<Skeleton className="h-4 w-24" />
							</TableCell>
							<TableCell>
								<Skeleton className="h-4 w-40" />
							</TableCell>
							<TableCell>
								<Skeleton className="h-6 w-20" />
							</TableCell>
							<TableCell>
								<Skeleton className="h-4 w-16" />
							</TableCell>
							<TableCell>
								<Skeleton className="h-4 w-32" />
							</TableCell>
							<TableCell>
								<Skeleton className="h-4 w-20" />
							</TableCell>
							<TableCell>
								<Skeleton className="size-7" />
							</TableCell>
						</TableRow>
					))}
				</TableBody>
			</Table>
			<div className="flex items-center justify-between">
				<Skeleton className="h-4 w-24" />
				<div className="flex items-center gap-1">
					<Skeleton className="size-7" />
					<Skeleton className="size-7" />
					<Skeleton className="size-7" />
					<Skeleton className="size-7" />
				</div>
			</div>
		</div>
	);
}
