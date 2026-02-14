import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, Store, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { ProductRetailerResponse } from "@/modules/products/types";

interface RetailerListProps {
	retailers: ProductRetailerResponse[];
	onRemove: (id: string) => void;
	isRemoving: boolean;
}

/** Extract display domain from a full URL */
function extractDomain(url: string): string {
	try {
		return new URL(url).hostname;
	} catch {
		return url;
	}
}

export function RetailerList({
	retailers,
	onRemove,
	isRemoving,
}: RetailerListProps) {
	if (retailers.length === 0) {
		return (
			<p className="py-4 text-center text-muted-foreground text-sm">
				No retailers added yet. Add a retailer URL to start tracking.
			</p>
		);
	}

	return (
		<div className="divide-y">
			{retailers.map((retailer) => (
				<div
					key={retailer.id}
					className="flex items-center justify-between gap-4 py-3"
				>
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
			))}
		</div>
	);
}
