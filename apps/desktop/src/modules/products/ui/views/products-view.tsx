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
import { MESSAGES } from "@/constants";
import { withToast, withToastVoid } from "@/lib/toast-helpers";
import { cn } from "@/lib/utils";
import { useCheckAllAvailability } from "@/modules/products/hooks/useAvailability";
import { useProductDialogs } from "@/modules/products/hooks/useProductDialogs";
import { useProducts } from "@/modules/products/hooks/useProducts";
import type { BulkCheckSummary } from "@/modules/products/types";
import { DeleteConfirmDialog } from "@/modules/products/ui/components/delete-confirm-dialog";
import { ProductFormDialog } from "@/modules/products/ui/components/product-form-dialog";
import { ProductsTable } from "@/modules/products/ui/components/products-table";
import { FullPageError } from "@/modules/shared/ui/components/full-page-error";

/** Configuration for create/edit dialog text and labels */
const DIALOG_CONFIG = {
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

/**
 * Formats the success message for bulk availability check results.
 *
 * Prioritizes showing "back in stock" count when products have returned
 * to availability, otherwise shows the success/total ratio.
 */
function formatCheckAllSuccessMessage(result: BulkCheckSummary): string {
	const baseMessage = MESSAGES.AVAILABILITY.CHECK_ALL_COMPLETE;
	if (result.back_in_stock_count > 0) {
		return `${baseMessage} - ${result.back_in_stock_count} product(s) back in stock!`;
	}
	return `${baseMessage} (${result.successful}/${result.total} successful)`;
}

/**
 * Validates that required form fields are present.
 * Shows an error toast if validation fails.
 *
 * @returns true if valid, false otherwise
 */
function validateFormData(formData: { name: string; url: string }): boolean {
	if (!formData.name || !formData.url) {
		toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
		return false;
	}
	return true;
}

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

	const { checkAllAvailability, isCheckingAll, progress } =
		useCheckAllAvailability();

	const {
		dialogState,
		openCreateDialog,
		openEditDialog,
		openDeleteDialog,
		closeDialog,
		updateFormData,
	} = useProductDialogs();

	const handleCreate = async () => {
		if (dialogState.type !== "create") return;
		const { formData } = dialogState;

		if (!validateFormData(formData)) return;

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

		if (!validateFormData(formData)) return;

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
			success: formatCheckAllSuccessMessage,
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
							className={cn("size-4", isCheckingAll && "animate-spin")}
						/>
						{isCheckingAll && progress
							? `Checking ${progress.currentIndex}/${progress.totalCount}...`
							: isCheckingAll
								? "Checking..."
								: "Check All"}
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

			{(dialogState.type === "create" || dialogState.type === "edit") && (
				<ProductFormDialog
					open={true}
					onOpenChange={(open) => !open && closeDialog()}
					title={DIALOG_CONFIG[dialogState.type].title}
					description={DIALOG_CONFIG[dialogState.type].description}
					formData={dialogState.formData}
					onFormChange={updateFormData}
					onSubmit={dialogState.type === "create" ? handleCreate : handleUpdate}
					isSubmitting={dialogState.type === "create" ? isCreating : isUpdating}
					submitLabel={DIALOG_CONFIG[dialogState.type].submitLabel}
					submittingLabel={DIALOG_CONFIG[dialogState.type].submittingLabel}
					idPrefix={dialogState.type}
				/>
			)}

			<DeleteConfirmDialog
				open={dialogState.type === "delete"}
				onOpenChange={(open) => !open && closeDialog()}
				productName={
					dialogState.type === "delete" ? dialogState.product.name : ""
				}
				onConfirm={handleDelete}
				isDeleting={isDeleting}
			/>
		</div>
	);
}
