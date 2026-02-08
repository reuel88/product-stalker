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
}

export function ProductFormDialog({
	open,
	onOpenChange,
	mode,
	formData,
	onFormChange,
	onSubmit,
	isSubmitting,
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
							value={formData.name}
							onChange={(e) =>
								onFormChange({ ...formData, name: e.target.value })
							}
							placeholder="Product name"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${mode}-url`}>URL</Label>
						<Input
							id={`${mode}-url`}
							value={formData.url}
							onChange={(e) =>
								onFormChange({ ...formData, url: e.target.value })
							}
							placeholder="https://example.com/product"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${mode}-description`}>Description</Label>
						<Textarea
							id={`${mode}-description`}
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
							value={formData.notes || ""}
							onChange={(e) =>
								onFormChange({ ...formData, notes: e.target.value })
							}
							placeholder="Optional notes"
						/>
					</div>
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
