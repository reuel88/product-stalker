import { beforeEach, describe, expect, it, vi } from "vitest";
import { COMMANDS, MESSAGES } from "@/constants";
import { SettingsSkeleton } from "@/modules/settings/ui/components/settings-skeleton";
import { SettingsComponent } from "@/modules/settings/ui/views/settings";
import {
	createMockSettings,
	createMockUpdateAvailable,
	createMockUpdateInfo,
} from "../../mocks/data";
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

describe("SettingsComponent", () => {
	beforeEach(() => {
		getMockedInvoke().mockReset();
		vi.clearAllMocks();
	});

	describe("loading state", () => {
		it("should show skeleton while loading", async () => {
			// Never resolve to keep loading state
			getMockedInvoke().mockImplementation(() => new Promise(() => {}));

			render(<SettingsComponent />);

			// SettingsComponent shows SettingsSkeleton when loading
			// The skeleton doesn't have the Settings title visible
			await waitFor(() => {
				expect(screen.queryByText("Settings")).not.toBeInTheDocument();
			});
		});
	});

	describe("error state", () => {
		it("should show error message when settings fail to load", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS) {
					return Promise.reject(new Error("Failed"));
				}
				if (cmd === COMMANDS.GET_CURRENT_VERSION) {
					return Promise.resolve("1.0.0");
				}
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Failed to load settings")).toBeInTheDocument();
			});
		});
	});

	describe("settings display", () => {
		it("should render all setting sections", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
			});

			render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Settings")).toBeInTheDocument();
			});

			expect(screen.getByText("Appearance")).toBeInTheDocument();
			expect(screen.getByText("System")).toBeInTheDocument();
			expect(screen.getByText("Logging")).toBeInTheDocument();
			expect(screen.getByText("Notifications")).toBeInTheDocument();
			expect(screen.getByText("Interface")).toBeInTheDocument();
			expect(screen.getByText("Updates")).toBeInTheDocument();
		});

		it("should display current version", async () => {
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.2.3",
			});

			render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("v1.2.3")).toBeInTheDocument();
			});
		});
	});

	describe("theme switching", () => {
		it("should change theme when selected", async () => {
			const settings = createMockSettings({ theme: "system" });
			const updatedSettings = createMockSettings({ theme: "dark" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Settings")).toBeInTheDocument();
			});

			// Find the theme selector (first combobox in Appearance section)
			const comboboxes = screen.getAllByRole("combobox");
			const themeSelect = comboboxes[0]; // First combobox is theme
			await user.click(themeSelect);

			// Select dark theme
			const darkOption = screen.getByRole("option", { name: "Dark" });
			await user.click(darkOption);

			await waitFor(() => {
				expect(mockSetTheme).toHaveBeenCalledWith("dark");
			});

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith(MESSAGES.SETTINGS.SAVED);
			});
		});
	});

	describe("toggle switches", () => {
		it("should toggle show_in_tray setting", async () => {
			const settings = createMockSettings({ show_in_tray: false });
			const updatedSettings = createMockSettings({ show_in_tray: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Show in tray")).toBeInTheDocument();
			});

			// Get all switches - show_in_tray is the first switch
			const switches = screen.getAllByRole("switch");
			await user.click(switches[0]);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { show_in_tray: true } },
				);
			});
		});

		it("should toggle enable_notifications setting", async () => {
			const settings = createMockSettings({ enable_notifications: true });
			const updatedSettings = createMockSettings({
				enable_notifications: false,
			});
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Enable notifications")).toBeInTheDocument();
			});

			// Get all switches
			// Order: show_in_tray, launch_at_login, enable_logging, enable_notifications, sidebar_expanded
			const switches = screen.getAllByRole("switch");
			await user.click(switches[3]);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { enable_notifications: false } },
				);
			});
		});

		it("should toggle launch_at_login setting", async () => {
			const settings = createMockSettings({ launch_at_login: false });
			const updatedSettings = createMockSettings({ launch_at_login: true });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Launch at login")).toBeInTheDocument();
			});

			// switch index 1 is launch_at_login
			const switches = screen.getAllByRole("switch");
			await user.click(switches[1]);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { launch_at_login: true } },
				);
			});
		});

		it("should toggle enable_logging setting", async () => {
			const settings = createMockSettings({ enable_logging: true });
			const updatedSettings = createMockSettings({ enable_logging: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Enable logging")).toBeInTheDocument();
			});

			// switch index 2 is enable_logging
			const switches = screen.getAllByRole("switch");
			await user.click(switches[2]);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { enable_logging: false } },
				);
			});
		});

		it("should toggle sidebar_expanded setting", async () => {
			const settings = createMockSettings({ sidebar_expanded: true });
			const updatedSettings = createMockSettings({ sidebar_expanded: false });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Sidebar expanded")).toBeInTheDocument();
			});

			// switch index 4 is sidebar_expanded
			const switches = screen.getAllByRole("switch");
			await user.click(switches[4]);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { sidebar_expanded: false } },
				);
			});
		});
	});

	describe("log level", () => {
		it("should change log level when selected", async () => {
			const settings = createMockSettings({
				log_level: "info",
				enable_logging: true,
			});
			const updatedSettings = createMockSettings({ log_level: "debug" });
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: settings,
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.UPDATE_SETTINGS]: updatedSettings,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Log level")).toBeInTheDocument();
			});

			// Find the log level combobox (second combobox after theme)
			const comboboxes = screen.getAllByRole("combobox");
			const logLevelSelect = comboboxes[1]; // Second combobox is log level
			await user.click(logLevelSelect);

			// Select debug level
			const debugOption = screen.getByRole("option", { name: "Debug" });
			await user.click(debugOption);

			await waitFor(() => {
				expect(getMockedInvoke()).toHaveBeenCalledWith(
					COMMANDS.UPDATE_SETTINGS,
					{ input: { log_level: "debug" } },
				);
			});
		});
	});

	describe("update checking", () => {
		it("should check for updates when button clicked", async () => {
			const noUpdate = createMockUpdateInfo();
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: noUpdate,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(toast.info).toHaveBeenCalledWith(
					"You're running the latest version",
				);
			});
		});

		it("should show update available toast", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			mockInvokeMultiple({
				[COMMANDS.GET_SETTINGS]: createMockSettings(),
				[COMMANDS.GET_CURRENT_VERSION]: "1.0.0",
				[COMMANDS.CHECK_FOR_UPDATE]: updateAvailable,
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(toast.success).toHaveBeenCalledWith("Update available: v2.0.0");
			});
		});

		it("should show error toast when check fails", async () => {
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.reject(new Error("Network error"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Failed to check for updates");
			});
		});
	});

	describe("update installation", () => {
		it("should show Update Now button when update available", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.resolve(updateAvailable);
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(screen.getByText("Update Now")).toBeInTheDocument();
			});

			expect(screen.getByText("v2.0.0")).toBeInTheDocument();
		});

		it("should install update when Update Now clicked", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.resolve(updateAvailable);
				if (cmd === COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE)
					return Promise.resolve(undefined);
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(screen.getByText("Update Now")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Update Now"));

			await waitFor(() => {
				expect(toast.info).toHaveBeenCalledWith("Downloading update...");
			});
		});

		it("should show error toast when install fails", async () => {
			const updateAvailable = createMockUpdateAvailable("2.0.0");
			getMockedInvoke().mockImplementation((cmd: string) => {
				if (cmd === COMMANDS.GET_SETTINGS)
					return Promise.resolve(createMockSettings());
				if (cmd === COMMANDS.GET_CURRENT_VERSION)
					return Promise.resolve("1.0.0");
				if (cmd === COMMANDS.CHECK_FOR_UPDATE)
					return Promise.resolve(updateAvailable);
				if (cmd === COMMANDS.DOWNLOAD_AND_INSTALL_UPDATE)
					return Promise.reject(new Error("Download failed"));
				return Promise.reject(new Error(`Unmocked: ${cmd}`));
			});

			const { user } = render(<SettingsComponent />);

			await waitFor(() => {
				expect(screen.getByText("Check for Updates")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Check for Updates"));

			await waitFor(() => {
				expect(screen.getByText("Update Now")).toBeInTheDocument();
			});

			await user.click(screen.getByText("Update Now"));

			await waitFor(() => {
				expect(toast.error).toHaveBeenCalledWith("Failed to install update");
			});
		});
	});
});

describe("SettingsSkeleton", () => {
	it("should render skeleton cards", () => {
		render(<SettingsSkeleton />);

		// The skeleton renders 5 cards using data-slot attribute
		const cards = document.querySelectorAll('[data-slot="card"]');
		expect(cards.length).toBe(5);
	});
});
