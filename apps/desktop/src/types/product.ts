export interface ProductResponse {
	id: string;
	name: string;
	url: string;
	description: string | null;
	notes: string | null;
	created_at: string;
	updated_at: string;
}
