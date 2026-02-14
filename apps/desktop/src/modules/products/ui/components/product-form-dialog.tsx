import { Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import type { RetailerEntry } from "@/modules/products/hooks/useProductDialogs";
import type { CreateProductInput } from "@/modules/products/hooks/useProducts";

const MODE_CONFIG = {
	create: {
		title: "Add Product",
		description: "Add a new product to track",
		submitLabel: "Create",
		submittingLabel: "Creating...",
	},
	edit: {
		title: "Edit Product",
		description: "Update product details",
		submitLabel: "Save",
		submittingLabel: "Saving...",
	},
} as const;

interface ProductFormDialogProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	mode: "create" | "edit";
	formData: CreateProductInput;
	onFormChange: (data: CreateProductInput) => void;
	onSubmit: () => void;
	isSubmitting: boolean;
	retailerEntries?: RetailerEntry[];
	onAddRetailerEntry?: () => void;
	onUpdateRetailerEntry?: (index: number, entry: RetailerEntry) => void;
	onRemoveRetailerEntry?: (index: number) => void;
}

export function ProductFormDialog({
	open,
	onOpenChange,
	mode,
	formData,
	onFormChange,
	onSubmit,
	isSubmitting,
	retailerEntries,
	onAddRetailerEntry,
	onUpdateRetailerEntry,
	onRemoveRetailerEntry,
}: ProductFormDialogProps) {
	const { title, description, submitLabel, submittingLabel } =
		MODE_CONFIG[mode];
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>{description}</DialogDescription>
				</DialogHeader>
				<div className="grid gap-4 py-4">
					<div className="grid gap-2">
						<Label htmlFor={`${mode}-name`}>Name</Label>
						<Input
							id={`${mode}-name`}
							data-testid="product-name-input"
							value={formData.name}
							onChange={(e) =>
								onFormChange({ ...formData, name: e.target.value })
							}
							placeholder="Product name"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${mode}-description`}>Description</Label>
						<Textarea
							id={`${mode}-description`}
							data-testid="product-description-input"
							value={formData.description || ""}
							onChange={(e) =>
								onFormChange({ ...formData, description: e.target.value })
							}
							placeholder="Optional description"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${mode}-notes`}>Notes</Label>
						<Textarea
							id={`${mode}-notes`}
							data-testid="product-notes-input"
							value={formData.notes || ""}
							onChange={(e) =>
								onFormChange({ ...formData, notes: e.target.value })
							}
							placeholder="Optional notes"
						/>
					</div>
					{mode === "create" && retailerEntries && (
						<div className="grid gap-2">
							<div className="flex items-center justify-between">
								<Label>Retailers</Label>
								<Button
									type="button"
									variant="outline"
									size="xs"
									onClick={onAddRetailerEntry}
								>
									<Plus />
									Add Retailer
								</Button>
							</div>
							{retailerEntries.length === 0 ? (
								<p className="text-muted-foreground text-sm">
									No retailers added. You can add retailers after creation too.
								</p>
							) : (
								<div className="grid gap-2">
									{retailerEntries.map((entry, index) => (
										<div key={entry.id} className="flex items-center gap-2">
											<Input
												data-testid={`retailer-url-${index}`}
												value={entry.url}
												onChange={(e) =>
													onUpdateRetailerEntry?.(index, {
														...entry,
														url: e.target.value,
													})
												}
												placeholder="https://example.com/product"
											/>
											<Input
												data-testid={`retailer-label-${index}`}
												value={entry.label}
												onChange={(e) =>
													onUpdateRetailerEntry?.(index, {
														...entry,
														label: e.target.value,
													})
												}
												placeholder="Label (optional)"
											/>
											<Button
												type="button"
												variant="ghost"
												size="icon-sm"
												data-testid={`retailer-remove-${index}`}
												onClick={() => onRemoveRetailerEntry?.(index)}
											>
												<Trash2 />
											</Button>
										</div>
									))}
								</div>
							)}
						</div>
					)}
				</div>
				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Cancel
					</Button>
					<Button onClick={onSubmit} disabled={isSubmitting}>
						{isSubmitting ? submittingLabel : submitLabel}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
