import { Plus } from "lucide-react";
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
import { MESSAGES } from "@/constants";
import {
	type CreateProductInput,
	useProducts,
} from "@/modules/products/hooks/useProducts";
import type { ProductResponse } from "@/modules/products/types";
import { ProductsTable } from "@/modules/products/ui/components/products-table";
import { ErrorState } from "@/modules/shared/ui/components/error-state";

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

	const [createDialogOpen, setCreateDialogOpen] = useState(false);
	const [editDialogOpen, setEditDialogOpen] = useState(false);
	const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
	const [selectedProduct, setSelectedProduct] =
		useState<ProductResponse | null>(null);

	const [formData, setFormData] = useState<CreateProductInput>({
		name: "",
		url: "",
		description: "",
		notes: "",
	});

	const resetForm = () => {
		setFormData({ name: "", url: "", description: "", notes: "" });
	};

	const handleCreate = async () => {
		if (!formData.name || !formData.url) {
			toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
			return;
		}

		try {
			await createProduct({
				name: formData.name,
				url: formData.url,
				description: formData.description || null,
				notes: formData.notes || null,
			});
			toast.success(MESSAGES.PRODUCT.CREATED);
			setCreateDialogOpen(false);
			resetForm();
		} catch {
			toast.error(MESSAGES.PRODUCT.CREATE_FAILED);
		}
	};

	const handleEdit = (product: ProductResponse) => {
		setSelectedProduct(product);
		setFormData({
			name: product.name,
			url: product.url,
			description: product.description || "",
			notes: product.notes || "",
		});
		setEditDialogOpen(true);
	};

	const handleUpdate = async () => {
		if (!selectedProduct) return;
		if (!formData.name || !formData.url) {
			toast.error(MESSAGES.VALIDATION.NAME_URL_REQUIRED);
			return;
		}

		try {
			await updateProduct({
				id: selectedProduct.id,
				input: {
					name: formData.name,
					url: formData.url,
					description: formData.description || null,
					notes: formData.notes || null,
				},
			});
			toast.success(MESSAGES.PRODUCT.UPDATED);
			setEditDialogOpen(false);
			setSelectedProduct(null);
			resetForm();
		} catch {
			toast.error(MESSAGES.PRODUCT.UPDATE_FAILED);
		}
	};

	const handleDeleteClick = (product: ProductResponse) => {
		setSelectedProduct(product);
		setDeleteDialogOpen(true);
	};

	const handleDelete = async () => {
		if (!selectedProduct) return;

		try {
			await deleteProduct(selectedProduct.id);
			toast.success(MESSAGES.PRODUCT.DELETED);
			setDeleteDialogOpen(false);
			setSelectedProduct(null);
		} catch {
			toast.error(MESSAGES.PRODUCT.DELETE_FAILED);
		}
	};

	if (error) {
		return (
			<div className="flex h-screen w-full flex-col items-center justify-center">
				<ErrorState
					title="Failed to load products"
					description="Please try again later"
				/>
			</div>
		);
	}

	return (
		<div className="container mx-auto max-w-4xl overflow-y-auto px-4 py-6">
			<div className="mb-6 flex items-center justify-between">
				<h1 className="font-semibold text-xl">Products</h1>
				<Button size="sm" onClick={() => setCreateDialogOpen(true)}>
					<Plus className="size-4" />
					Add Product
				</Button>
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
						onEdit={handleEdit}
						onDelete={handleDeleteClick}
					/>
				</CardContent>
			</Card>

			{/* Create Dialog */}
			<Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
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
						<Button
							variant="outline"
							onClick={() => setCreateDialogOpen(false)}
						>
							Cancel
						</Button>
						<Button onClick={handleCreate} disabled={isCreating}>
							{isCreating ? "Creating..." : "Create"}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>

			{/* Edit Dialog */}
			<Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Edit Product</DialogTitle>
						<DialogDescription>Update product details</DialogDescription>
					</DialogHeader>
					<div className="grid gap-4 py-4">
						<div className="grid gap-2">
							<Label htmlFor="edit-name">Name</Label>
							<Input
								id="edit-name"
								value={formData.name}
								onChange={(e) =>
									setFormData({ ...formData, name: e.target.value })
								}
								placeholder="Product name"
							/>
						</div>
						<div className="grid gap-2">
							<Label htmlFor="edit-url">URL</Label>
							<Input
								id="edit-url"
								value={formData.url}
								onChange={(e) =>
									setFormData({ ...formData, url: e.target.value })
								}
								placeholder="https://example.com/product"
							/>
						</div>
						<div className="grid gap-2">
							<Label htmlFor="edit-description">Description</Label>
							<Textarea
								id="edit-description"
								value={formData.description || ""}
								onChange={(e) =>
									setFormData({ ...formData, description: e.target.value })
								}
								placeholder="Optional description"
							/>
						</div>
						<div className="grid gap-2">
							<Label htmlFor="edit-notes">Notes</Label>
							<Textarea
								id="edit-notes"
								value={formData.notes || ""}
								onChange={(e) =>
									setFormData({ ...formData, notes: e.target.value })
								}
								placeholder="Optional notes"
							/>
						</div>
					</div>
					<DialogFooter>
						<Button variant="outline" onClick={() => setEditDialogOpen(false)}>
							Cancel
						</Button>
						<Button onClick={handleUpdate} disabled={isUpdating}>
							{isUpdating ? "Saving..." : "Save"}
						</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>

			{/* Delete Confirmation Dialog */}
			<Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Delete Product</DialogTitle>
						<DialogDescription>
							Are you sure you want to delete "{selectedProduct?.name}"? This
							action cannot be undone.
						</DialogDescription>
					</DialogHeader>
					<DialogFooter>
						<Button
							variant="outline"
							onClick={() => setDeleteDialogOpen(false)}
						>
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
