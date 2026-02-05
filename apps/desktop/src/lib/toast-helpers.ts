import { toast } from "sonner";

type ToastMessages = {
	success: string;
	error: string;
};

/**
 * Execute an async operation with automatic toast notifications.
 *
 * Shows a success toast on completion or an error toast on failure.
 * Returns the result on success, or undefined on failure.
 *
 * @example
 * ```ts
 * const result = await withToast(
 *   () => createProduct(data),
 *   { success: "Product created", error: "Failed to create product" }
 * );
 * if (result) closeDialog();
 * ```
 */
export async function withToast<T>(
	operation: () => Promise<T>,
	messages: ToastMessages,
): Promise<T | undefined> {
	try {
		const result = await operation();
		toast.success(messages.success);
		return result;
	} catch {
		toast.error(messages.error);
		return undefined;
	}
}
