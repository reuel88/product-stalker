import { invoke } from "@tauri-apps/api/core";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { COMMANDS } from "./constants";
import { earlyApplyPalette } from "./modules/shared/themes/apply-palette";

earlyApplyPalette();

const rootElement = document.getElementById("app");

if (!rootElement) {
	throw new Error("Root element not found");
}

if (!rootElement.innerHTML) {
	const root = ReactDOM.createRoot(rootElement);
	root.render(<App />);
	invoke(COMMANDS.CLOSE_SPLASHSCREEN).catch(console.error);
}
