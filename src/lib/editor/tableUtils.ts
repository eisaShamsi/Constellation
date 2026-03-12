/**
 * Table parsing and manipulation utilities for markdown tables in CodeMirror.
 */
import type { EditorState } from '@codemirror/state';

export interface ParsedTable {
	startLine: number;
	endLine: number;
	rows: string[][];
	separatorLineNum: number;
	alignments: ('left' | 'center' | 'right' | 'none')[];
	columnCount: number;
	cursorRow: number;
	cursorCol: number;
}

/** Check if a line looks like a table separator row */
function isSeparatorRow(text: string): boolean {
	return /^\|?[\s:]*-{2,}[\s:]*(\|[\s:]*-{2,}[\s:]*)*\|?\s*$/.test(text.trim());
}

/** Check if a line looks like a table row */
function isTableRow(text: string): boolean {
	const trimmed = text.trim();
	return trimmed.includes('|') && !isSeparatorRow(trimmed);
}

/** Parse cells from a table row */
function parseCells(text: string): string[] {
	let t = text.trim();
	if (t.startsWith('|')) t = t.slice(1);
	if (t.endsWith('|')) t = t.slice(0, -1);
	return t.split('|').map(c => c.trim());
}

/** Parse alignment from separator row */
function parseAlignments(text: string): ('left' | 'center' | 'right' | 'none')[] {
	return parseCells(text).map(cell => {
		const c = cell.trim();
		const left = c.startsWith(':');
		const right = c.endsWith(':');
		if (left && right) return 'center';
		if (right) return 'right';
		if (left) return 'left';
		return 'none';
	});
}

/** Determine which cell column the cursor is in */
function getCursorColumn(lineText: string, cursorOffset: number): number {
	let col = 0;
	let inCell = false;
	for (let i = 0; i < cursorOffset && i < lineText.length; i++) {
		if (lineText[i] === '|') {
			if (inCell) col++;
			inCell = true;
		}
	}
	// If line starts with |, first | is not a separator but the start
	if (lineText.trimStart().startsWith('|')) {
		return Math.max(0, col - 1);
	}
	return col;
}

/** Parse a markdown table around the given cursor position */
export function parseTable(state: EditorState, pos: number): ParsedTable | null {
	const cursorLine = state.doc.lineAt(pos);
	const lineText = cursorLine.text.trim();

	// Must have a pipe to be in a table
	if (!lineText.includes('|')) return null;

	// Find table boundaries by scanning up and down
	let startLineNum = cursorLine.number;
	let endLineNum = cursorLine.number;

	// Scan up
	for (let i = cursorLine.number - 1; i >= 1; i--) {
		const line = state.doc.line(i);
		const t = line.text.trim();
		if (isTableRow(t) || isSeparatorRow(t)) {
			startLineNum = i;
		} else break;
	}

	// Scan down
	for (let i = cursorLine.number + 1; i <= state.doc.lines; i++) {
		const line = state.doc.line(i);
		const t = line.text.trim();
		if (isTableRow(t) || isSeparatorRow(t)) {
			endLineNum = i;
		} else break;
	}

	// Must have at least 3 rows (header + separator + 1 data row or just header + separator)
	if (endLineNum - startLineNum < 1) return null;

	// Find separator row
	let separatorLineNum = -1;
	for (let i = startLineNum; i <= endLineNum; i++) {
		if (isSeparatorRow(state.doc.line(i).text)) {
			separatorLineNum = i;
			break;
		}
	}
	if (separatorLineNum === -1) return null;

	// Parse all rows
	const rows: string[][] = [];
	let maxCols = 0;
	for (let i = startLineNum; i <= endLineNum; i++) {
		if (i === separatorLineNum) continue;
		const cells = parseCells(state.doc.line(i).text);
		rows.push(cells);
		maxCols = Math.max(maxCols, cells.length);
	}

	// Normalize row lengths
	for (const row of rows) {
		while (row.length < maxCols) row.push('');
	}

	const alignments = parseAlignments(state.doc.line(separatorLineNum).text);
	while (alignments.length < maxCols) alignments.push('none');

	// Calculate cursor position in table
	const cursorRow = cursorLine.number < separatorLineNum
		? cursorLine.number - startLineNum
		: cursorLine.number - startLineNum - 1; // Subtract 1 for separator
	const cursorCol = getCursorColumn(cursorLine.text, pos - cursorLine.from);

	return {
		startLine: startLineNum,
		endLine: endLineNum,
		rows,
		separatorLineNum,
		alignments,
		columnCount: maxCols,
		cursorRow: Math.max(0, Math.min(cursorRow, rows.length - 1)),
		cursorCol: Math.max(0, Math.min(cursorCol, maxCols - 1)),
	};
}

/** Format a table back to markdown string */
export function formatTable(rows: string[][], alignments: ('left' | 'center' | 'right' | 'none')[]): string {
	const colCount = Math.max(rows.length > 0 ? Math.max(...rows.map(r => r.length)) : 0, alignments.length);

	// Calculate column widths
	const widths: number[] = [];
	for (let c = 0; c < colCount; c++) {
		let max = 3; // minimum width for separator
		for (const row of rows) {
			max = Math.max(max, (row[c] || '').length);
		}
		widths.push(max);
	}

	const lines: string[] = [];

	// Header row (first row)
	if (rows.length > 0) {
		lines.push(formatRow(rows[0], widths, colCount));
	}

	// Separator row
	const sep = widths.map((w, i) => {
		const a = alignments[i] || 'none';
		const dashes = '-'.repeat(w);
		if (a === 'center') return ':' + dashes.slice(1, -1) + ':';
		if (a === 'right') return dashes.slice(0, -1) + ':';
		if (a === 'left') return ':' + dashes.slice(1);
		return dashes;
	});
	lines.push('| ' + sep.join(' | ') + ' |');

	// Data rows
	for (let i = 1; i < rows.length; i++) {
		lines.push(formatRow(rows[i], widths, colCount));
	}

	return lines.join('\n');
}

function formatRow(row: string[], widths: number[], colCount: number): string {
	const cells: string[] = [];
	for (let c = 0; c < colCount; c++) {
		cells.push((row[c] || '').padEnd(widths[c]));
	}
	return '| ' + cells.join(' | ') + ' |';
}

/** Add a new row after the specified index */
export function addRow(table: ParsedTable, afterRow: number): ParsedTable {
	const newRow = new Array(table.columnCount).fill('');
	const insertIdx = Math.min(afterRow + 1, table.rows.length);
	const newRows = [...table.rows];
	newRows.splice(insertIdx, 0, newRow);
	return { ...table, rows: newRows, cursorRow: insertIdx };
}

/** Add a new column after the specified index */
export function addColumn(table: ParsedTable, afterCol: number): ParsedTable {
	const insertIdx = Math.min(afterCol + 1, table.columnCount);
	const newRows = table.rows.map(row => {
		const r = [...row];
		r.splice(insertIdx, 0, '');
		return r;
	});
	const newAlignments = [...table.alignments];
	newAlignments.splice(insertIdx, 0, 'none');
	return { ...table, rows: newRows, alignments: newAlignments, columnCount: table.columnCount + 1, cursorCol: insertIdx };
}

/** Delete a row */
export function deleteRow(table: ParsedTable, row: number): ParsedTable | null {
	if (table.rows.length <= 1) return null; // Can't delete header
	if (row === 0) return null; // Don't delete header
	const newRows = table.rows.filter((_, i) => i !== row);
	return { ...table, rows: newRows, cursorRow: Math.min(row, newRows.length - 1) };
}

/** Delete a column */
export function deleteColumn(table: ParsedTable, col: number): ParsedTable | null {
	if (table.columnCount <= 1) return null;
	const newRows = table.rows.map(row => row.filter((_, i) => i !== col));
	const newAlignments = table.alignments.filter((_, i) => i !== col);
	return { ...table, rows: newRows, alignments: newAlignments, columnCount: table.columnCount - 1, cursorCol: Math.min(col, table.columnCount - 2) };
}

/** Set alignment for a column */
export function setAlignment(table: ParsedTable, col: number, alignment: 'left' | 'center' | 'right'): ParsedTable {
	const newAlignments = [...table.alignments];
	newAlignments[col] = alignment;
	return { ...table, alignments: newAlignments };
}

/** Get the position of a specific cell in the formatted table for cursor placement */
export function getCellPosition(formattedTable: string, row: number, col: number): number {
	const lines = formattedTable.split('\n');
	// Row 0 = line 0, then separator = line 1, row 1+ = line row+1
	const lineIdx = row === 0 ? 0 : row + 1;
	if (lineIdx >= lines.length) return formattedTable.length;
	const line = lines[lineIdx];
	// Find the start of the target column
	let pipeCount = 0;
	for (let i = 0; i < line.length; i++) {
		if (line[i] === '|') {
			if (pipeCount === col + 1) return i - 1; // End of previous cell
			pipeCount++;
			if (pipeCount === col + 1) {
				// Skip space after pipe
				return i + 2;
			}
		}
	}
	// Offset to reach the correct line
	let offset = 0;
	for (let i = 0; i < lineIdx; i++) {
		offset += lines[i].length + 1;
	}
	return offset + 2; // After "| "
}
