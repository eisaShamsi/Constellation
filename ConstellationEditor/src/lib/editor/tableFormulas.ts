/**
 * Table formula parser and evaluator for markdown tables.
 * Supports: =SUM(), =AVG(), =COUNT(), =MIN(), =MAX()
 * Cell references: A1, B2, etc. Ranges: A1:A5, B2:D2
 */

export type FormulaResult = { value: string; error?: string };

/** Column letter to 0-based index */
function colToIndex(col: string): number {
	let idx = 0;
	for (const ch of col.toUpperCase()) {
		idx = idx * 26 + (ch.charCodeAt(0) - 64);
	}
	return idx - 1;
}

/** 0-based index to column letter */
export function indexToCol(idx: number): string {
	let s = '';
	idx++;
	while (idx > 0) {
		idx--;
		s = String.fromCharCode(65 + (idx % 26)) + s;
		idx = Math.floor(idx / 26);
	}
	return s;
}

/** Parse a cell reference like "A1" → { col: 0, row: 0 } (row is 0-based data row, not header) */
function parseCellRef(ref: string): { col: number; row: number } | null {
	const m = ref.match(/^([A-Za-z]+)(\d+)$/);
	if (!m) return null;
	return { col: colToIndex(m[1]), row: parseInt(m[2], 10) - 1 };
}

/** Expand a range "A1:B3" into individual cell references */
function expandRange(range: string): { col: number; row: number }[] {
	const parts = range.split(':');
	if (parts.length !== 2) return [];
	const start = parseCellRef(parts[0].trim());
	const end = parseCellRef(parts[1].trim());
	if (!start || !end) return [];
	const cells: { col: number; row: number }[] = [];
	const minCol = Math.min(start.col, end.col);
	const maxCol = Math.max(start.col, end.col);
	const minRow = Math.min(start.row, end.row);
	const maxRow = Math.max(start.row, end.row);
	for (let r = minRow; r <= maxRow; r++) {
		for (let c = minCol; c <= maxCol; c++) {
			cells.push({ col: c, row: r });
		}
	}
	return cells;
}

/** Get numeric value from a cell, data rows only (row 0 = first data row after header) */
function getCellValue(rows: string[][], col: number, row: number): number | null {
	// rows[0] is header, data starts at rows[1]
	const dataRow = row + 1; // formula row 1 = rows[1] (first data row)
	if (dataRow < 1 || dataRow >= rows.length) return null;
	if (col < 0 || col >= (rows[dataRow]?.length ?? 0)) return null;
	const raw = rows[dataRow][col].trim();
	if (raw === '') return null;
	const num = parseFloat(raw);
	return isNaN(num) ? null : num;
}

/** Resolve argument (cell ref, range, or literal number) to array of numbers */
function resolveArg(arg: string, rows: string[][]): number[] {
	const trimmed = arg.trim();
	// Range: A1:B3
	if (trimmed.includes(':')) {
		const cells = expandRange(trimmed);
		const nums: number[] = [];
		for (const c of cells) {
			const v = getCellValue(rows, c.col, c.row);
			if (v !== null) nums.push(v);
		}
		return nums;
	}
	// Single cell ref
	const ref = parseCellRef(trimmed);
	if (ref) {
		const v = getCellValue(rows, ref.col, ref.row);
		return v !== null ? [v] : [];
	}
	// Literal number
	const n = parseFloat(trimmed);
	return isNaN(n) ? [] : [n];
}

/** Evaluate a formula string like "=SUM(A1:A5)" */
export function evaluateFormula(formula: string, rows: string[][]): FormulaResult {
	const trimmed = formula.trim();
	if (!trimmed.startsWith('=')) return { value: formula };

	const expr = trimmed.slice(1).trim();
	const funcMatch = expr.match(/^(\w+)\((.+)\)$/);
	if (!funcMatch) return { value: formula, error: 'Invalid formula' };

	const funcName = funcMatch[1].toUpperCase();
	const argsStr = funcMatch[2];
	// Split args by comma, but respect ranges (no nested parens to worry about)
	const argParts = argsStr.split(',');
	const allNums: number[] = [];
	for (const part of argParts) {
		allNums.push(...resolveArg(part, rows));
	}

	if (allNums.length === 0 && funcName !== 'COUNT') {
		return { value: '0' };
	}

	switch (funcName) {
		case 'SUM': {
			const sum = allNums.reduce((a, b) => a + b, 0);
			return { value: formatNum(sum) };
		}
		case 'AVG':
		case 'AVERAGE': {
			if (allNums.length === 0) return { value: '0' };
			const avg = allNums.reduce((a, b) => a + b, 0) / allNums.length;
			return { value: formatNum(avg) };
		}
		case 'COUNT': {
			return { value: String(allNums.length) };
		}
		case 'MIN': {
			return { value: formatNum(Math.min(...allNums)) };
		}
		case 'MAX': {
			return { value: formatNum(Math.max(...allNums)) };
		}
		default:
			return { value: formula, error: `Unknown function: ${funcName}` };
	}
}

function formatNum(n: number): string {
	if (Number.isInteger(n)) return String(n);
	return n.toFixed(2).replace(/\.?0+$/, '');
}

/** Check if a cell contains a formula */
export function isFormula(cell: string): boolean {
	return cell.trim().startsWith('=') && /^=\w+\(/.test(cell.trim());
}

/** Process all formulas in a table, returning new rows with evaluated values.
 *  Original formula text is preserved; this returns display values. */
export function evaluateTableFormulas(rows: string[][]): string[][] {
	return rows.map(row =>
		row.map(cell => {
			if (isFormula(cell)) {
				const result = evaluateFormula(cell, rows);
				return result.error ? cell : result.value;
			}
			return cell;
		})
	);
}
