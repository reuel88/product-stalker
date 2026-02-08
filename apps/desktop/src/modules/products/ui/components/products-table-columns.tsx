import { Link } from "@tanstack/react-router";
import type { ColumnDef } from "@tanstack/react-table";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, MoreHorizontal, Pencil, Trash2 } from "lucide-react";

import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { UI } from "@/constants";
import type { ProductResponse } from "@/modules/products/types";

/** Renders text truncated to maxLength with a title tooltip showing full text */
function TruncatedText({
	text,
	maxLength,
	className,
	testId,
}: {
	text: string;
	maxLength: number;
	className?: string;
	testId?: string;
}) {
	const truncated =
		text.length > maxLength ? `${text.slice(0, maxLength)}...` : text;
	return (
		<span data-testid={testId} className={className} title={text}>
			{truncated}
		</span>
	);
}

interface ColumnOptions {
	onEdit?: (product: ProductResponse) => void;
	onDelete?: (product: ProductResponse) => void;
	AvailabilityCell: React.ComponentType<Record<string, never>>;
	PriceCell: React.ComponentType<{ productId: string }>;
}

export function createProductColumns(
	options: ColumnOptions,
): ColumnDef<ProductResponse>[] {
	const { onEdit, onDelete, AvailabilityCell, PriceCell } = options;

	return [
		{
			accessorKey: "name",
			header: "Name",
			cell: ({ row }) => (
				<Link
					to="/products/$id"
					params={{ id: row.original.id }}
					className="font-medium text-primary hover:underline"
				>
					{row.getValue("name")}
				</Link>
			),
		},
		{
			accessorKey: "url",
			header: "URL",
			cell: ({ row }) => {
				const url = row.original.url;
				return (
					<button
						type="button"
						onClick={() => openUrl(url)}
						className="inline-flex items-center gap-1 text-left text-primary hover:underline"
					>
						<TruncatedText text={url} maxLength={UI.TRUNCATE.URL_LENGTH} />
						<ExternalLink className="size-3" />
					</button>
				);
			},
		},
		{
			id: "availability",
			header: "Availability",
			cell: () => <AvailabilityCell />,
		},
		{
			id: "price",
			header: "Price",
			cell: ({ row }) => <PriceCell productId={row.original.id} />,
		},
		{
			accessorKey: "description",
			header: "Description",
			cell: ({ row }) => {
				const description = row.original.description;
				if (!description)
					return (
						<span
							data-testid={`description-${row.original.id}`}
							className="text-muted-foreground"
						>
							-
						</span>
					);
				return (
					<TruncatedText
						text={description}
						maxLength={UI.TRUNCATE.DESCRIPTION_LENGTH}
						testId={`description-${row.original.id}`}
					/>
				);
			},
		},
		{
			accessorKey: "created_at",
			header: "Created",
			cell: ({ row }) => {
				const date = new Date(row.original.created_at);
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
}
