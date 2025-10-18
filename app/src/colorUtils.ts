import { getStorageItem, setStorageItem } from "./storageUtils.ts";

type Palette = Record<string, string>;

interface Theme {
	accent: Palette;
	background: Palette;
}

const defaultBaseColor = "#b34472";

const hexToRgb = (hex: string): [number, number, number] => {
	// Handle CSS variables that might return rgb() format
	if (hex.startsWith("rgb")) {
		const rgbValues = hex.match(/\d+/g);
		if (rgbValues && rgbValues.length >= 3) {
			return [
				parseInt(rgbValues[0]),
				parseInt(rgbValues[1]),
				parseInt(rgbValues[2]),
			];
		}
	}

	// Handle hex format
	const cleanHex = hex.startsWith("#") ? hex.slice(1) : hex;
	const r = parseInt(cleanHex.slice(0, 2), 16);
	const g = parseInt(cleanHex.slice(2, 4), 16);
	const b = parseInt(cleanHex.slice(4, 6), 16);
	return [r, g, b];
};

const rgbToHex = (r: number, g: number, b: number): string => {
	return `#${Math.round(r).toString(16).padStart(2, "0")}${
		Math.round(g).toString(16).padStart(2, "0")
	}${Math.round(b).toString(16).padStart(2, "0")}`;
};

const generateAccentPalette = (baseColor: string): Palette => {
	// Lighten or darken a color by a factor
	const adjustColor = (
		rgb: [number, number, number],
		factor: number,
	): string => {
		const [r, g, b] = rgb;
		if (factor > 1) {
			// Lighten
			return rgbToHex(
				r + (255 - r) * (factor - 1),
				g + (255 - g) * (factor - 1),
				b + (255 - b) * (factor - 1),
			);
		} else {
			// Darken
			return rgbToHex(r * factor, g * factor, b * factor);
		}
	};

	const baseRgb = hexToRgb(baseColor);

	// Adjustment factors for each shade level
	const factors = {
		"50": 1.9, // Lightest
		"100": 1.7,
		"200": 1.5,
		"300": 1.3,
		"400": 1.1,
		"500": 1.0, // Base color
		"600": 0.85,
		"700": 0.7,
		"800": 0.55,
		"900": 0.4,
		"950": 0.25, // Darkest
	};

	const palette: Palette = {};
	for (const [shade, factor] of Object.entries(factors)) {
		palette[shade] = adjustColor(baseRgb, factor);
	}

	return palette;
};

const generateBackgroundPalette = (
	baseColor: string,
): Palette => {
	const baseRgb = hexToRgb(baseColor);
	const backgroundPalette: Palette = {};

	// Create neutral shades from white to black with varying color hints
	const shadeValues = {
		"50": { grayValue: 255, intensity: 0.05 },
		"100": { grayValue: 245, intensity: 0.05 },
		"200": { grayValue: 225, intensity: 0.05 },
		"300": { grayValue: 205, intensity: 0.05 },
		"400": { grayValue: 185, intensity: 0.05 },
		"500": { grayValue: 105, intensity: 0.03 },
		"600": { grayValue: 85, intensity: 0.03 },
		"700": { grayValue: 65, intensity: 0.03 },
		"800": { grayValue: 45, intensity: 0.03 },
		"900": { grayValue: 25, intensity: 0.03 },
		"950": { grayValue: 15, intensity: 0.03 },
	};

	for (
		const [shade, { grayValue, intensity }] of Object.entries(shadeValues)
	) {
		// Add a hint of the base color
		const r = grayValue + (baseRgb[0] - grayValue) * intensity;
		const g = grayValue + (baseRgb[1] - grayValue) * intensity;
		const b = grayValue + (baseRgb[2] - grayValue) * intensity;

		backgroundPalette[shade] = rgbToHex(r, g, b);
	}

	return backgroundPalette;
};

const generateTheme = (baseColor: string): Theme => ({
	accent: generateAccentPalette(baseColor),
	background: generateBackgroundPalette(baseColor),
});

const setBaseColor = (baseColor: string) =>
	setStorageItem("user-base-color", baseColor);

export const getBaseColor = () => {
	const baseColor = getStorageItem<string>("user-base-color");
	if (baseColor) return baseColor;

	setBaseColor(defaultBaseColor);
	return defaultBaseColor;
};

const setTheme = (theme: Theme) => setStorageItem("user-theme", theme);

export const getTheme = (): Theme => {
	const theme = getStorageItem<Theme>("user-theme");
	if (theme) return theme;

	// If no theme exists, generate from stored base color or default
	const baseColor = getBaseColor();
	const newTheme = generateTheme(baseColor);
	setTheme(newTheme);
	return newTheme;
};

export const generateThemeCSS = (theme: Theme): string => {
	const cssVars: string[] = [];

	for (const [shade, color] of Object.entries(theme.accent)) {
		cssVars.push(`--color-accent-${shade}: ${color};`);
	}

	for (const [shade, color] of Object.entries(theme.background)) {
		cssVars.push(`--color-background-${shade}: ${color};`);
	}

	return `:root { ${cssVars.join(" ")} }`;
};

const applyTheme = (theme: Theme) => {
	// Apply accent palette
	for (const [shade, color] of Object.entries(theme.accent)) {
		document.documentElement.style.setProperty(
			`--color-accent-${shade}`,
			color,
		);
	}

	// Apply background palette
	for (const [shade, color] of Object.entries(theme.background)) {
		document.documentElement.style.setProperty(
			`--color-background-${shade}`,
			color,
		);
	}
};

export const updateTheme = (baseColor: string) => {
	const newTheme = generateTheme(baseColor);

	// Apply and store only the base color and theme
	applyTheme(newTheme);
	setBaseColor(baseColor);
	setTheme(newTheme);
};

export const resetTheme = () => {
	const defaultTheme = generateTheme(defaultBaseColor);

	// Apply and store only the base color and theme
	applyTheme(defaultTheme);
	setBaseColor(defaultBaseColor);
	setTheme(defaultTheme);
};
