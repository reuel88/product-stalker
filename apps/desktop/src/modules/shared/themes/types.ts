export type PaletteId = "default" | "ocean" | "rose";

export type CssVarMap = Record<string, string>;

export interface PaletteDefinition {
	id: PaletteId;
	name: string;
	description: string;
	preview: {
		primary: string;
		accent: string;
		background: string;
	};
	light: CssVarMap;
	dark: CssVarMap;
}
