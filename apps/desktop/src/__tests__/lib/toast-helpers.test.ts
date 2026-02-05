import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { withToast } from "@/lib/toast-helpers";

vi.mock("sonner", () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn(),
	},
}));

describe("withToast utility", () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it("should return result and show success toast on successful operation", async () => {
		const mockResult = { id: "123", name: "Test" };
		const operation = vi.fn().mockResolvedValue(mockResult);

		const result = await withToast(operation, {
			success: "Operation succeeded",
			error: "Operation failed",
		});

		expect(result).toEqual(mockResult);
		expect(operation).toHaveBeenCalledOnce();
		expect(toast.success).toHaveBeenCalledWith("Operation succeeded");
		expect(toast.error).not.toHaveBeenCalled();
	});

	it("should return undefined and show error toast on failed operation", async () => {
		const operation = vi.fn().mockRejectedValue(new Error("Network error"));

		const result = await withToast(operation, {
			success: "Operation succeeded",
			error: "Operation failed",
		});

		expect(result).toBeUndefined();
		expect(operation).toHaveBeenCalledOnce();
		expect(toast.error).toHaveBeenCalledWith("Operation failed");
		expect(toast.success).not.toHaveBeenCalled();
	});

	it("should handle void operations", async () => {
		const operation = vi.fn().mockResolvedValue(undefined);

		const result = await withToast(operation, {
			success: "Deleted successfully",
			error: "Delete failed",
		});

		expect(result).toBeUndefined();
		expect(toast.success).toHaveBeenCalledWith("Deleted successfully");
	});

	it("should handle operations that return falsy values", async () => {
		const operation = vi.fn().mockResolvedValue(null);

		const result = await withToast(operation, {
			success: "Success",
			error: "Error",
		});

		expect(result).toBeNull();
		expect(toast.success).toHaveBeenCalledWith("Success");
	});
});
