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

interface ProductFormDialogProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	title: string;
	description: string;
	formData: CreateProductInput;
	onFormChange: (data: CreateProductInput) => void;
	onSubmit: () => void;
	isSubmitting: boolean;
	submitLabel: string;
	submittingLabel: string;
	idPrefix: string;
}

export function ProductFormDialog({
	open,
	onOpenChange,
	title,
	description,
	formData,
	onFormChange,
	onSubmit,
	isSubmitting,
	submitLabel,
	submittingLabel,
	idPrefix,
}: ProductFormDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>{title}</DialogTitle>
					<DialogDescription>{description}</DialogDescription>
				</DialogHeader>
				<div className="grid gap-4 py-4">
					<div className="grid gap-2">
						<Label htmlFor={`${idPrefix}-name`}>Name</Label>
						<Input
							id={`${idPrefix}-name`}
							value={formData.name}
							onChange={(e) =>
								onFormChange({ ...formData, name: e.target.value })
							}
							placeholder="Product name"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${idPrefix}-url`}>URL</Label>
						<Input
							id={`${idPrefix}-url`}
							value={formData.url}
							onChange={(e) =>
								onFormChange({ ...formData, url: e.target.value })
							}
							placeholder="https://example.com/product"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${idPrefix}-description`}>Description</Label>
						<Textarea
							id={`${idPrefix}-description`}
							value={formData.description || ""}
							onChange={(e) =>
								onFormChange({ ...formData, description: e.target.value })
							}
							placeholder="Optional description"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor={`${idPrefix}-notes`}>Notes</Label>
						<Textarea
							id={`${idPrefix}-notes`}
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
