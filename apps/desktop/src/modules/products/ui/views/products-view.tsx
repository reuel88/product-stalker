import { invoke } from "@tauri-apps/api/core";
import { Plus, RefreshCw } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { COMMANDS, MESSAGES } from "@/constants";
import { withToast, withToastVoid } from "@/lib/toast-helpers";
import { cn } from "@/lib/utils";
import {
	useCheckAllAvailability,
	useManualVerificationListener,
} from "@/modules/products/hooks/useAvailability";
import { useProductDialogs } from "@/modules/products/hooks/useProductDialogs";
import { useProducts } from "@/modules/products/hooks/useProducts";
import type {
	BulkCheckSummary,
	ProductRetailerResponse,
} from "@/modules/products/types";
import { DeleteConfirmDialog } from "@/modules/products/ui/components/delete-confirm-dialog";
import { ProductFormDialog } from "@/modules/products/ui/components/product-form-dialog";
import { ProductsTable } from "@/modules/products/ui/components/products-table";
import { FullPageError } from "@/modules/shared/ui/components/full-page-error";

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
 * Returns the appropriate button text based on the check state.
 *
 * Three states: idle, checking without progress, checking with progress.
 */
function getCheckButtonText(
	isChecking: boolean,
	progress: { currentIndex: number; totalCount: number } | null,
): string {
	if (!isChecking) return "Check All";
	if (progress)
		return `Checking ${progress.currentIndex}/${progress.totalCount}...`;
	return "Checking...";
}

/**
 * Validates that required form fields are present.
 * Shows an error toast if validation fails.
 *
 * @returns true if valid, false otherwise
 */
function isFormDataValid(formData: { name: string }): boolean {
	if (!formData.name) {
		toast.error(MESSAGES.VALIDATION.NAME_REQUIRED);
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
		addRetailerEntry,
		updateRetailerEntry,
		removeRetailerEntry,
	} = useProductDialogs();

	const [isSubmittingWithRetailers, setIsSubmittingWithRetailers] =
		useState(false);

	// Auto-subscribe to manual verification events
	useManualVerificationListener();

	const handleSubmit = async () => {
		if (dialogState.type !== "create" && dialogState.type !== "edit") return;
		const { formData } = dialogState;

		if (!isFormDataValid(formData)) return;

		const input = {
			name: formData.name,
			description: formData.description || null,
			notes: formData.notes || null,
		};

		if (dialogState.type === "edit") {
			const result = await withToast(
				() => updateProduct({ id: dialogState.product.id, input }),
				{
					success: MESSAGES.PRODUCT.UPDATED,
					error: MESSAGES.PRODUCT.UPDATE_FAILED,
				},
			);
			if (result) closeDialog();
			return;
		}

		// Create mode: create product, then add retailers
		const retailerEntries = dialogState.retailerEntries.filter(
			(e) => e.url.trim() !== "",
		);

		setIsSubmittingWithRetailers(true);
		try {
			const product = await withToast(() => createProduct(input), {
				success: MESSAGES.PRODUCT.CREATED,
				error: MESSAGES.PRODUCT.CREATE_FAILED,
			});
			if (!product) return;

			for (const entry of retailerEntries) {
				try {
					await invoke<ProductRetailerResponse>(COMMANDS.ADD_PRODUCT_RETAILER, {
						input: {
							product_id: product.id,
							url: entry.url.trim(),
							label: entry.label.trim() || null,
						},
					});
				} catch {
					toast.error(`Failed to add retailer: ${entry.url}`);
				}
			}

			closeDialog();
		} finally {
			setIsSubmittingWithRetailers(false);
		}
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
		await withToast(checkAllAvailability, {
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
						{getCheckButtonText(isCheckingAll, progress)}
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
					mode={dialogState.type}
					formData={dialogState.formData}
					onFormChange={updateFormData}
					onSubmit={handleSubmit}
					isSubmitting={
						dialogState.type === "create"
							? isCreating || isSubmittingWithRetailers
							: isUpdating
					}
					{...(dialogState.type === "create" && {
						retailerEntries: dialogState.retailerEntries,
						onAddRetailerEntry: addRetailerEntry,
						onUpdateRetailerEntry: updateRetailerEntry,
						onRemoveRetailerEntry: removeRetailerEntry,
					})}
				/>
			)}

			{dialogState.type === "delete" && (
				<DeleteConfirmDialog
					open={true}
					onOpenChange={(open) => !open && closeDialog()}
					productName={dialogState.product.name}
					onConfirm={handleDelete}
					isDeleting={isDeleting}
				/>
			)}
		</div>
	);
}
