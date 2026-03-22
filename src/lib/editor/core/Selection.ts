/**
 * Selection model for the Constellation Editor.
 *
 * anchor = where the selection started
 * head   = where the cursor is (extends from anchor)
 * When anchor === head, it's a caret (no selection).
 */

export interface SelectionState {
	anchor: number;
	head: number;
}

export class EditorSelection {
	anchor: number;
	head: number;

	constructor(anchor: number = 0, head?: number) {
		this.anchor = anchor;
		this.head = head ?? anchor;
	}

	/** Whether this is a collapsed cursor (no range). */
	get isCollapsed(): boolean {
		return this.anchor === this.head;
	}

	/** Start of the selection (min of anchor/head). */
	get from(): number {
		return Math.min(this.anchor, this.head);
	}

	/** End of the selection (max of anchor/head). */
	get to(): number {
		return Math.max(this.anchor, this.head);
	}

	/** Length of the selected text. */
	get length(): number {
		return this.to - this.from;
	}

	/** Create a collapsed cursor at offset. */
	static cursor(offset: number): EditorSelection {
		return new EditorSelection(offset, offset);
	}

	/** Create a range selection. */
	static range(anchor: number, head: number): EditorSelection {
		return new EditorSelection(anchor, head);
	}

	/** Clone this selection. */
	clone(): EditorSelection {
		return new EditorSelection(this.anchor, this.head);
	}

	/** Check equality. */
	equals(other: EditorSelection): boolean {
		return this.anchor === other.anchor && this.head === other.head;
	}

	/** Clamp selection to valid range. */
	clamp(maxOffset: number): EditorSelection {
		return new EditorSelection(
			Math.max(0, Math.min(this.anchor, maxOffset)),
			Math.max(0, Math.min(this.head, maxOffset)),
		);
	}

	/** Serialize to plain object. */
	toJSON(): SelectionState {
		return { anchor: this.anchor, head: this.head };
	}

	/** Deserialize from plain object. */
	static fromJSON(state: SelectionState): EditorSelection {
		return new EditorSelection(state.anchor, state.head);
	}
}
