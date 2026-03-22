/**
 * Undo/redo history for the Constellation Editor.
 *
 * Groups rapid keystrokes into a single undo step (300ms threshold).
 * Each entry stores the change and the selection before/after.
 */

import type { TextChange } from './PieceTable';
import type { SelectionState } from './Selection';

export interface HistoryEntry {
	changes: TextChange[];
	selectionBefore: SelectionState;
	selectionAfter: SelectionState;
	timestamp: number;
}

const GROUP_TIMEOUT = 300; // ms — group rapid edits into one undo step
const MAX_HISTORY = 500;

export class History {
	private undoStack: HistoryEntry[] = [];
	private redoStack: HistoryEntry[] = [];

	/**
	 * Push a new edit. If it's within GROUP_TIMEOUT of the last edit,
	 * merge into the same undo step.
	 */
	push(change: TextChange, selectionBefore: SelectionState, selectionAfter: SelectionState): void {
		const now = Date.now();
		this.redoStack.length = 0; // Clear redo on new edit

		const last = this.undoStack[this.undoStack.length - 1];
		if (last && now - last.timestamp < GROUP_TIMEOUT && this.canMerge(last, change)) {
			last.changes.push(change);
			last.selectionAfter = selectionAfter;
			last.timestamp = now;
		} else {
			this.undoStack.push({
				changes: [change],
				selectionBefore,
				selectionAfter,
				timestamp: now,
			});
			if (this.undoStack.length > MAX_HISTORY) {
				this.undoStack.shift();
			}
		}
	}

	/**
	 * Force a boundary — next edit won't merge with previous.
	 */
	pushBoundary(): void {
		const last = this.undoStack[this.undoStack.length - 1];
		if (last) last.timestamp = 0;
	}

	/**
	 * Pop the last undo entry.
	 */
	undo(): HistoryEntry | null {
		const entry = this.undoStack.pop();
		if (entry) this.redoStack.push(entry);
		return entry ?? null;
	}

	/**
	 * Pop the last redo entry.
	 */
	redo(): HistoryEntry | null {
		const entry = this.redoStack.pop();
		if (entry) this.undoStack.push(entry);
		return entry ?? null;
	}

	get canUndo(): boolean {
		return this.undoStack.length > 0;
	}

	get canRedo(): boolean {
		return this.redoStack.length > 0;
	}

	clear(): void {
		this.undoStack.length = 0;
		this.redoStack.length = 0;
	}

	private canMerge(last: HistoryEntry, change: TextChange): boolean {
		// Only merge simple single-character insertions or deletions
		if (last.changes.length > 20) return false;
		const prev = last.changes[last.changes.length - 1];
		// Merge consecutive inserts
		if (prev.oldLength === 0 && change.oldLength === 0) {
			return change.offset === prev.offset + prev.newLength;
		}
		// Merge consecutive backspaces
		if (prev.newLength === 0 && change.newLength === 0) {
			return change.offset === prev.offset - change.oldLength || change.offset === prev.offset;
		}
		return false;
	}
}
