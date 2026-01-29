import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	getPaginationRowModel,
	useReactTable,
} from "@tanstack/react-table";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
	ChevronFirst,
	ChevronLast,
	ChevronLeft,
	ChevronRight,
	ExternalLink,
	MoreHorizontal,
	Pencil,
	Trash2,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
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
import type { ProductResponse } from "@/modules/products/types";

interface ProductsTableProps {
	products: ProductResponse[];
	isLoading?: boolean;
	onEdit?: (product: ProductResponse) => void;
	onDelete?: (product: ProductResponse) => void;
}

export function ProductsTable({
	products,
	isLoading,
	onEdit,
	onDelete,
}: ProductsTableProps) {
	const columns: ColumnDef<ProductResponse>[] = [
		{
			accessorKey: "name",
			header: "Name",
			cell: ({ row }) => (
				<span className="font-medium">{row.getValue("name")}</span>
			),
		},
		{
			accessorKey: "url",
			header: "URL",
			cell: ({ row }) => {
				const url = row.getValue("url") as string;
				const truncated =
					url.length > UI.TRUNCATE.URL_LENGTH
						? `${url.slice(0, UI.TRUNCATE.URL_LENGTH)}...`
						: url;
				return (
					<button
						type="button"
						onClick={() => openUrl(url)}
						className="inline-flex items-center gap-1 text-primary hover:underline"
						title={url}
					>
						{truncated}
						<ExternalLink className="size-3" />
					</button>
				);
			},
		},
		{
			accessorKey: "description",
			header: "Description",
			cell: ({ row }) => {
				const description = row.getValue("description") as string | null;
				if (!description)
					return <span className="text-muted-foreground">-</span>;
				const truncated =
					description.length > UI.TRUNCATE.DESCRIPTION_LENGTH
						? `${description.slice(0, UI.TRUNCATE.DESCRIPTION_LENGTH)}...`
						: description;
				return <span title={description}>{truncated}</span>;
			},
		},
		{
			accessorKey: "created_at",
			header: "Created",
			cell: ({ row }) => {
				const date = new Date(row.getValue("created_at") as string);
				return <span>{date.toLocaleDateString()}</span>;
			},
		},
		{
			id: "actions",
			header: () => <span className="sr-only">Actions</span>,
			cell: ({ row }) => {
				const product = row.original;
				return (
					<DropdownMenu>
						<DropdownMenuTrigger className="inline-flex size-7 items-center justify-center rounded-none hover:bg-muted">
							<MoreHorizontal className="size-4" />
							<span className="sr-only">Open menu</span>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end">
							<DropdownMenuItem onClick={() => onEdit?.(product)}>
								<Pencil />
								Edit
							</DropdownMenuItem>
							<DropdownMenuItem
								variant="destructive"
								onClick={() => onDelete?.(product)}
							>
								<Trash2 />
								Delete
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				);
			},
		},
	];

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
