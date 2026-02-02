export interface ProductResponse {
	id: string;
	name: string;
	url: string;
	description: string | null;
	notes: string | null;
	created_at: string;
	updated_at: string;
}

export type AvailabilityStatus =
	| "in_stock"
	| "out_of_stock"
	| "back_order"
	| "unknown";

export interface AvailabilityCheckResponse {
	id: string;
	product_id: string;
	status: AvailabilityStatus;
	raw_availability: string | null;
	error_message: string | null;
	checked_at: string;
}
