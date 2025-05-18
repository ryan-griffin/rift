interface Theme {
	accent: Record<string, string>;
	background: Record<string, string>;
}

function getDefaultBaseColor(): string {
	const cssDefault = getComputedStyle(document.documentElement)
		.getPropertyValue("--default-base-color")
		.trim();

	return cssDefault;
}

export function storeBaseColor(baseColor: string) {
	localStorage.setItem("user-base-color", baseColor);
}

export function getBaseColor(): string | null {
	return localStorage.getItem("user-base-color");
}

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

function generateAccentPalette(baseColor: string): Record<string, string> {
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

	const palette: Record<string, string> = {};
	for (const [shade, factor] of Object.entries(factors)) {
		palette[shade] = adjustColor(baseRgb, factor);
	}

	return palette;
}

function generateBackgroundPalette(baseColor: string): Record<string, string> {
	const baseRgb = hexToRgb(baseColor);
	const backgroundPalette: Record<string, string> = {};

	// Create neutral shades from white to black with varying color hints
	const shadeValues = {
		"50": { grayValue: 255, intensity: 0.05 },
		"100": { grayValue: 245, intensity: 0.1 },
		"200": { grayValue: 235, intensity: 0.1 },
		"300": { grayValue: 225, intensity: 0.1 },
		"400": { grayValue: 185, intensity: 0.1 },
		"500": { grayValue: 125, intensity: 0.05 },
		"600": { grayValue: 100, intensity: 0.05 },
		"700": { grayValue: 75, intensity: 0.05 },
		"800": { grayValue: 50, intensity: 0.05 },
		"900": { grayValue: 25, intensity: 0.05 },
		"950": { grayValue: 0, intensity: 0.1 },
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
}

export function generateTheme(baseColor: string): Theme {
	return {
		accent: generateAccentPalette(baseColor),
		background: generateBackgroundPalette(baseColor),
	};
}

export function applyTheme(theme: Theme) {
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
}

// Store the complete theme
export function storeTheme(theme: Theme) {
	localStorage.setItem("user-theme", JSON.stringify(theme));
}

// Get the stored theme or generate a new one
export function getTheme(): Theme {
	const themeJson = localStorage.getItem("user-theme");

	if (themeJson) {
		try {
			return JSON.parse(themeJson);
		} catch (e) {
			console.error("Failed to parse saved theme", e);
		}
	}

	// If no theme exists, generate from stored base color or default
	const baseColor = getBaseColor() || getDefaultBaseColor();
	const newTheme = generateTheme(baseColor);
	storeBaseColor(baseColor);
	storeTheme(newTheme);
	return newTheme;
}

export function initializeTheme() {
	const theme = getTheme();
	applyTheme(theme);
}

export function updateTheme(baseColor: string) {
	const newTheme = generateTheme(baseColor);

	// Apply and store only the base color and theme
	applyTheme(newTheme);
	storeBaseColor(baseColor);
	storeTheme(newTheme);
}

export function resetTheme() {
	const defaultColor = getDefaultBaseColor();
	const defaultTheme = generateTheme(defaultColor);

	// Apply and store only the base color and theme
	applyTheme(defaultTheme);
	storeBaseColor(defaultColor);
	storeTheme(defaultTheme);
}
