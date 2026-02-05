import { Button } from "@/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";

export interface DeleteConfirmDialogProps {
	open: boolean;
	onOpenChange: (open: boolean) => void;
	productName: string;
	onConfirm: () => void;
	isDeleting: boolean;
}

/**
 * A confirmation dialog for deleting a product.
 *
 * Displays a warning message with the product name and provides
 * cancel/delete actions. The delete button shows a loading state
 * while the deletion is in progress.
 */
export function DeleteConfirmDialog({
	open,
	onOpenChange,
	productName,
	onConfirm,
	isDeleting,
}: DeleteConfirmDialogProps) {
	return (
		<Dialog open={open} onOpenChange={onOpenChange}>
			<DialogContent>
				<DialogHeader>
					<DialogTitle>Delete Product</DialogTitle>
					<DialogDescription>
						Are you sure you want to delete "{productName}"? This action cannot
						be undone.
					</DialogDescription>
				</DialogHeader>
				<DialogFooter>
					<Button variant="outline" onClick={() => onOpenChange(false)}>
						Cancel
					</Button>
					<Button
						variant="destructive"
						onClick={onConfirm}
						disabled={isDeleting}
					>
						{isDeleting ? "Deleting..." : "Delete"}
					</Button>
				</DialogFooter>
			</DialogContent>
		</Dialog>
	);
}
