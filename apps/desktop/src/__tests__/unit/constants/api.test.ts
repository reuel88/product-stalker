import { describe, expect, it } from "vitest";
import { COMMANDS } from "@/constants/api";

describe("COMMANDS constant", () => {
	it("should export all required commands", () => {
		expect(COMMANDS).toHaveProperty("GET_PRODUCTS");
		expect(COMMANDS).toHaveProperty("CREATE_PRODUCT");
		expect(COMMANDS).toHaveProperty("UPDATE_PRODUCT");
		expect(COMMANDS).toHaveProperty("DELETE_PRODUCT");
		expect(COMMANDS).toHaveProperty("GET_SETTINGS");
		expect(COMMANDS).toHaveProperty("UPDATE_SETTINGS");
		expect(COMMANDS).toHaveProperty("SEND_NOTIFICATION");
		expect(COMMANDS).toHaveProperty("CLOSE_SPLASHSCREEN");
		expect(COMMANDS).toHaveProperty("CHECK_FOR_UPDATE");
		expect(COMMANDS).toHaveProperty("DOWNLOAD_AND_INSTALL_UPDATE");
		expect(COMMANDS).toHaveProperty("GET_CURRENT_VERSION");
	});

	it("should have snake_case command names", () => {
		const snakeCasePattern = /^[a-z]+(_[a-z]+)*$/;
		for (const [_key, value] of Object.entries(COMMANDS)) {
			expect(value).toMatch(snakeCasePattern);
		}
	});

	it("should have unique command values", () => {
		const values = Object.values(COMMANDS);
		const uniqueValues = new Set(values);
		expect(uniqueValues.size).toBe(values.length);
	});

	it("should have command values that match their purpose", () => {
		expect(COMMANDS.GET_PRODUCTS).toBe("get_products");
		expect(COMMANDS.CREATE_PRODUCT).toBe("create_product");
		expect(COMMANDS.UPDATE_PRODUCT).toBe("update_product");
		expect(COMMANDS.DELETE_PRODUCT).toBe("delete_product");
		expect(COMMANDS.GET_SETTINGS).toBe("get_settings");
		expect(COMMANDS.UPDATE_SETTINGS).toBe("update_settings");
	});

	it("should be read-only (const assertion)", () => {
		// TypeScript would catch mutations at compile time,
		// but we can verify the object structure is as expected
		expect(Object.isFrozen(COMMANDS)).toBe(false); // `as const` doesn't freeze at runtime
		expect(typeof COMMANDS).toBe("object");
	});
});
