import { type ComponentProps, useState } from "react";
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
import type { CreateProductInput } from "../../hooks/useProducts";

type CreateProductDialogProps = ComponentProps<typeof Dialog> & {
	onOpenChange: (open: boolean) => void;
};

export const CreateProductDialog = ({
	open,
	onOpenChange,
}: CreateProductDialogProps) => {
	const [formData, setFormData] = useState<CreateProductInput>({
		name: "",
		url: "",
		description: "",
		notes: "",
	});

	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Add Product</DialogTitle>
					<DialogDescription>Add a new product to track</DialogDescription>
				</DialogHeader>
				<div className="grid gap-4 py-4">
					<div className="grid gap-2">
						<Label htmlFor="create-name">Name</Label>
						<Input
							id="create-name"
							value={formData.name}
							onChange={(e) =>
								setFormData({ ...formData, name: e.target.value })
							}
							placeholder="Product name"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor="create-url">URL</Label>
						<Input
							id="create-url"
							value={formData.url}
							onChange={(e) =>
								setFormData({ ...formData, url: e.target.value })
							}
							placeholder="https://example.com/product"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor="create-description">Description</Label>
						<Textarea
							id="create-description"
							value={formData.description || ""}
							onChange={(e) =>
								setFormData({ ...formData, description: e.target.value })
							}
							placeholder="Optional description"
						/>
					</div>
					<div className="grid gap-2">
						<Label htmlFor="create-notes">Notes</Label>
						<Textarea
							id="create-notes"
							value={formData.notes || ""}
							onChange={(e) =>
								setFormData({ ...formData, notes: e.target.value })
							}
							placeholder="Optional notes"
						/>
					</div>
				</div>
				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Cancel
					</Button>
					{/* <Button onClick={handleCreate} disabled={isCreating}>
						{isCreating ? "Creating..." : "Create"}
					</Button> */}
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
};
