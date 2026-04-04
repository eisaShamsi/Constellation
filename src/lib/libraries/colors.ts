/**
 * Library color palette — shared across main window and second screen.
 * One source of truth. Import and use everywhere.
 */

export const LIBRARY_COLORS = [
	'#7c3aed', '#3b82f6', '#10b981', '#f59e0b', '#ef4444',
	'#ec4899', '#06b6d4', '#8b5cf6', '#84cc16', '#f97316'
];

/** Build a { libraryName → color } map from a list of libraries */
export function buildLibraryColorMap(libraries: { name: string }[]): Record<string, string> {
	const map: Record<string, string> = {};
	libraries.forEach((lib, i) => {
		map[lib.name] = LIBRARY_COLORS[i % LIBRARY_COLORS.length];
	});
	return map;
}
