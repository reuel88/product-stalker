import { Plus, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { MESSAGES } from "@/constants";
import { withToast, withToastVoid } from "@/lib/toast-helpers";
import { useCheckAllAvailability } from "@/modules/products/hooks/useAvailability";
import { useProductDialogs } from "@/modules/products/hooks/useProductDialogs";
import { useProducts } from "@/modules/products/hooks/useProducts";
import { ProductFormDialog } from "@/modules/products/ui/components/product-form-dialog";
import { ProductsTable } from "@/modules/products/ui/components/products-table";
import { FullPageError } from "@/modules/shared/ui/components/full-page-error";

export function ProductsView() {
	const {
		products,
		isLoading,
		error,
		createProduct,
		isCreating,
		updateProduct,
		isUpdating,
		deleteProduct,
		isDeleting,
	} = useProducts();

	const { checkAllAvailability, isCheckingAll } = useCheckAllAvailability();

	const {
		dialogState,
		openCreateDialog,
		openEditDialog,
		openDeleteDialog,
		closeDialog,
		updateFormData,
		initialFormData,
	} = useProductDialogs();

	const handleCreate = async () => {
		if (dialogState.type !== "create") return;
		const { formData } = dialogState;

		if (!formData.name || !formData.url) {
			toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
			return;
		}

		const result = await withToast(
			() =>
				createProduct({
					name: formData.name,
					url: formData.url,
					description: formData.description || null,
					notes: formData.notes || null,
				}),
			{
				success: MESSAGES.PRODUCT.CREATED,
				error: MESSAGES.PRODUCT.CREATE_FAILED,
			},
		);
		if (result) closeDialog();
	};

	const handleUpdate = async () => {
		if (dialogState.type !== "edit") return;
		const { product, formData } = dialogState;

		if (!formData.name || !formData.url) {
			toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
			return;
		}

		const result = await withToast(
			() =>
				updateProduct({
					id: product.id,
					input: {
						name: formData.name,
						url: formData.url,
						description: formData.description || null,
						notes: formData.notes || null,
					},
				}),
			{
				success: MESSAGES.PRODUCT.UPDATED,
				error: MESSAGES.PRODUCT.UPDATE_FAILED,
			},
		);
		if (result) closeDialog();
	};

	const handleDelete = async () => {
		if (dialogState.type !== "delete") return;

		const success = await withToastVoid(
			() => deleteProduct(dialogState.product.id),
			{
				success: MESSAGES.PRODUCT.DELETED,
				error: MESSAGES.PRODUCT.DELETE_FAILED,
			},
		);
		if (success) closeDialog();
	};

	const handleCheckAll = async () => {
		await withToast(() => checkAllAvailability(), {
			success: (result) => {
				if (result.back_in_stock_count > 0) {
					return `${MESSAGES.AVAILABILITY.CHECK_ALL_COMPLETE} - ${result.back_in_stock_count} product(s) back in stock!`;
				}
				return `${MESSAGES.AVAILABILITY.CHECK_ALL_COMPLETE} (${result.successful}/${result.total} successful)`;
			},
			error: MESSAGES.AVAILABILITY.CHECK_ALL_FAILED,
		});
	};

	if (error) {
		return (
			<FullPageError
				title="Failed to load products"
				description="Please try again later"
			/>
		);
	}

	return (
		<div className="container mx-auto overflow-y-auto px-4 py-6">
			<div className="mb-6 flex items-center justify-between">
				<h1 className="font-semibold text-xl">Products</h1>
				<div className="flex gap-2">
					<Button
						variant="outline"
						size="sm"
						onClick={handleCheckAll}
						disabled={isCheckingAll || !products?.length}
					>
						<RefreshCw
							className={`size-4 ${isCheckingAll ? "animate-spin" : ""}`}
						/>
						{isCheckingAll ? "Checking..." : "Check All"}
					</Button>
					<Button size="sm" onClick={openCreateDialog}>
						<Plus className="size-4" />
						Add Product
					</Button>
				</div>
			</div>

			<Card>
				<CardHeader>
					<CardTitle>All Products</CardTitle>
					<CardDescription>
						View and manage your tracked products
					</CardDescription>
				</CardHeader>
				<CardContent>
					<ProductsTable
						products={products ?? []}
						isLoading={isLoading}
						onEdit={openEditDialog}
						onDelete={openDeleteDialog}
					/>
				</CardContent>
			</Card>

			{/* Create Dialog */}
			<ProductFormDialog
				open={dialogState.type === "create"}
				onOpenChange={(open) => !open && closeDialog()}
				title="Add Product"
				description="Add a new product to track"
				formData={
					dialogState.type === "create" ? dialogState.formData : initialFormData
				}
				onFormChange={updateFormData}
				onSubmit={handleCreate}
				isSubmitting={isCreating}
				submitLabel="Create"
				submittingLabel="Creating..."
				idPrefix="create"
			/>

			{/* Edit Dialog */}
			<ProductFormDialog
				open={dialogState.type === "edit"}
				onOpenChange={(open) => !open && closeDialog()}
				title="Edit Product"
				description="Update product details"
				formData={
					dialogState.type === "edit" ? dialogState.formData : initialFormData
				}
				onFormChange={updateFormData}
				onSubmit={handleUpdate}
				isSubmitting={isUpdating}
				submitLabel="Save"
				submittingLabel="Saving..."
				idPrefix="edit"
			/>

			{/* Delete Confirmation Dialog */}
			<Dialog
				open={dialogState.type === "delete"}
				onOpenChange={(open) => !open && closeDialog()}
			>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Delete Product</DialogTitle>
						<DialogDescription>
							Are you sure you want to delete "
							{dialogState.type === "delete" ? dialogState.product.name : ""}
							"? This action cannot be undone.
						</DialogDescription>
					</DialogHeader>
					<DialogFooter>
						<Button variant="outline" onClick={closeDialog}>
							Cancel
						</Button>
						<Button
							variant="destructive"
							onClick={handleDelete}
							disabled={isDeleting}
						>
							{isDeleting ? "Deleting..." : "Delete"}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</div>
	);
}
