import { createFileRoute } from "@tanstack/react-router";

import { ProductDetailView } from "@/modules/products/ui/views/product-detail-view";

export const Route = createFileRoute("/products_/$id")({
	component: ProductDetailPage,
});

function ProductDetailPage() {
	const { id } = Route.useParams();
	return <ProductDetailView productId={id} />;
}
