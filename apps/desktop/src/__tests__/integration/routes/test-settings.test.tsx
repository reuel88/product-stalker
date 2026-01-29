import { beforeEach, describe, expect, it, vi } from "vitest";
import { COMMANDS } from "@/constants";
import { TestSettingsComponent } from "@/modules/settings/ui/views/test-settings";
import { createMockSettings } from "../../mocks/data";
import { getMockedInvoke, mockInvokeMultiple } from "../../mocks/tauri";
import { render, screen, waitFor } from "../../test-utils";

// Mock sonner toast
vi.mock("sonner", () => ({
	toast: {
		success: vi.fn(),
		error: vi.fn(),
		info: vi.fn(),
	},
}));

// Mock next-themes
const mockSetTheme = vi.fn();
vi.mock("next-themes", () => ({
	useTheme: () => ({
		theme: "system",
		setTheme: mockSetTheme,
	}),
}));

import { toast } from "sonner";

describe("TestSettingsComponent", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
		vi.clearAllMocks();
	});

	describe("loading state", () => {
		it("should show loading message while loading", async () => {
			getMockedInvoke().mockImplementation(() => new Promise(() => {}));

			render(<TestSettingsComponent />);

			expect(screen.getByText("Loading settings...")).toBeInTheDocument();
		});
	});

	describe("error state", () => {
		it("should show error message when settings fail to load", async () => {
			getMockedInvoke().mockRejectedValue(new Error("Failed"));

			render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Failed to load settings")).toBeInTheDocument();
			});
		});
	});

	describe("settings display", () => {
		it("should render all test cards", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
			});

			render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Settings Feature Tests")).toBeInTheDocument();
			});

			expect(screen.getByText("Current Settings State")).toBeInTheDocument();
			expect(screen.getByText("Theme Tests")).toBeInTheDocument();
			expect(screen.getByText("Boolean Settings Tests")).toBeInTheDocument();
			expect(screen.getByText("Log Level Tests")).toBeInTheDocument();
			expect(screen.getByText("Notification Tests")).toBeInTheDocument();
			expect(screen.getByText("Test Results Log")).toBeInTheDocument();
		});

		it("should display current settings as JSON", async () => {
			const settings = createMockSettings({ theme: "dark" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
			});

			render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText(/"theme": "dark"/)).toBeInTheDocument();
			});
		});
	});

	describe("theme tests", () => {
		it("should change theme when Set Light is clicked", async () => {
			const settings = createMockSettings({ theme: "system" });
			const updatedSettings = createMockSettings({ theme: "light" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Set Light")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Set Light"));

			await waitFor(() => {
				expect(mockSetTheme).toHaveBeenCalledWith("light");
			});

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith("Theme set to light");
			});
		});

		it("should change theme when Set Dark is clicked", async () => {
			const settings = createMockSettings({ theme: "system" });
			const updatedSettings = createMockSettings({ theme: "dark" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Set Dark")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Set Dark"));

			await waitFor(() => {
				expect(mockSetTheme).toHaveBeenCalledWith("dark");
			});
		});

		it("should show error when theme change fails", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.UPDATE_SETTINGS)
					return Promise.reject(new Error("Failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Set Light")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Set Light"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Failed to update theme");
			});
		});
	});

	describe("boolean settings tests", () => {
		it("should toggle show_in_tray setting", async () => {
			const settings = createMockSettings({ show_in_tray: false });
			const updatedSettings = createMockSettings({ show_in_tray: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Toggle show_in_tray")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Toggle show_in_tray"));

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{
						input: { show_in_tray: true },
					},
				);
			});

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith("show_in_tray updated");
			});
		});

		it("should show error when toggle fails", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.UPDATE_SETTINGS)
					return Promise.reject(new Error("Failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Toggle show_in_tray")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Toggle show_in_tray"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Update failed");
			});
		});
	});

	describe("log level tests", () => {
		it("should change log level when clicked", async () => {
			const settings = createMockSettings({ log_level: "info" });
			const updatedSettings = createMockSettings({ log_level: "debug" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("debug")).toBeInTheDocument();
			});

			await user.click(screen.getByText("debug"));

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{
						input: { log_level: "debug" },
					},
				);
			});

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith("Log level set to debug");
			});
		});

		it("should show error when log level change fails", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.UPDATE_SETTINGS)
					return Promise.reject(new Error("Failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("debug")).toBeInTheDocument();
			});

			await user.click(screen.getByText("debug"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Failed to update log level");
			});
		});
	});

	describe("notification tests", () => {
		it("should send test notification when button clicked", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.SEND_NOTIFICATION]: true,
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Send Test Notification")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Send Test Notification"));

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.SEND_NOTIFICATION,
					{
						input: {
							title: "Test Notification",
							body: "This is a test notification from Product Stalker!",
						},
					},
				);
			});

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith("Notification sent");
			});
		});

		it("should show info toast when notifications disabled", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings({
					enable_notifications: false,
				}),
				[COMMANDS.SEND_NOTIFICATION]: false,
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Send Test Notification")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Send Test Notification"));

			await waitFor(() => {
				expect(toast.info).toHaveBeenCalledWith("Notifications are disabled");
			});
		});

		it("should show error toast when notification fails", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.SEND_NOTIFICATION)
					return Promise.reject(new Error("Failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Send Test Notification")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Send Test Notification"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Failed to send notification");
			});
		});
	});

	describe("test log", () => {
		it("should show empty log message initially", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
			});

			render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("No test results yet")).toBeInTheDocument();
			});
		});

		it("should clear log when Clear Log clicked", async () => {
			const settings = createMockSettings();
			const updatedSettings = createMockSettings({ theme: "dark" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<TestSettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Set Dark")).toBeInTheDocument();
			});

			// Trigger an action to add a log entry
			await user.click(screen.getByText("Set Dark"));

			await waitFor(() => {
				expect(screen.getByText(/Theme set to dark/)).toBeInTheDocument();
			});

			// Clear the log
			await user.click(screen.getByText("Clear Log"));

			await waitFor(() => {
				expect(screen.getByText("No test results yet")).toBeInTheDocument();
			});
		});
	});
});
