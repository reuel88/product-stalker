import { toast } from "sonner";

type ToastMessages<T> = {
	success: string | ((result: T) => string);
	error: string;
};

type ToastMessagesVoid = {
	success: string;
	error: string;
};

/**
 * Execute an async operation with automatic toast notifications.
 *
 * Shows a success toast on completion or an error toast on failure.
 * Returns the result on success, or undefined on failure.
 *
 * The success message can be either a static string or a function that
 * receives the operation result to generate a dynamic message.
 *
 * @example
 * ```ts
 * // Static message
 * const result = await withToast(
 *   () => createProduct(data),
 *   { success: "Product created", error: "Failed to create product" }
 * );
 *
 * // Dynamic message based on result
 * const summary = await withToast(
 *   () => checkAllProducts(),
 *   {
 *     success: (result) => `Checked ${result.total} products`,
 *     error: "Failed to check products"
 *   }
 * );
 * ```
 */
export async function withToast<T>(
	operation: () => Promise<T>,
	messages: ToastMessages<T>,
): Promise<T | undefined> {
	try {
		const result = await operation();
		const successMessage =
			typeof messages.success === "function"
				? messages.success(result)
				: messages.success;
		toast.success(successMessage);
		return result;
	} catch {
		toast.error(messages.error);
		return undefined;
	}
}

/**
 * Execute a void async operation with automatic toast notifications.
 *
 * Use this for operations that don't return a value (e.g., delete).
 * Returns true on success, false on failure.
 */
export async function withToastVoid(
	operation: () => Promise<void>,
	messages: ToastMessagesVoid,
): Promise<boolean> {
	try {
		await operation();
		toast.success(messages.success);
		return true;
	} catch {
		toast.error(messages.error);
		return false;
	}
}
