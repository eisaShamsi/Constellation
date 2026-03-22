/**
 * WikilinkAutocomplete — [[note]] suggestions for the Constellation Editor.
 *
 * Triggers when the user types [[ and shows a filtered list of note names.
 */

export interface WikilinkSuggestion {
	name: string;
	path: string;
	libraryName?: string;
}

export class WikilinkAutocomplete {
	private notes: WikilinkSuggestion[] = [];
	private active = false;
	private query = '';
	private filteredNotes: WikilinkSuggestion[] = [];
	private selectedIndex = 0;
	private onComplete: ((name: string) => void) | null = null;

	setNotes(notes: WikilinkSuggestion[]): void {
		this.notes = notes;
	}

	/** Check if we should start autocomplete based on text before cursor. */
	checkTrigger(textBeforeCursor: string): boolean {
		const triggerIdx = textBeforeCursor.lastIndexOf('[[');
		if (triggerIdx < 0) {
			this.active = false;
			return false;
		}

		// Make sure there's no closing ]] between trigger and cursor
		const afterTrigger = textBeforeCursor.substring(triggerIdx + 2);
		if (afterTrigger.includes(']]')) {
			this.active = false;
			return false;
		}

		this.active = true;
		this.query = afterTrigger;
		this.filter();
		return true;
	}

	private filter(): void {
		const q = this.query.toLowerCase();
		this.filteredNotes = this.notes
			.filter(n => n.name.toLowerCase().includes(q))
			.slice(0, 10);
		this.selectedIndex = 0;
	}

	isActive(): boolean {
		return this.active;
	}

	getResults(): WikilinkSuggestion[] {
		return this.filteredNotes;
	}

	getSelectedIndex(): number {
		return this.selectedIndex;
	}

	moveUp(): void {
		if (this.selectedIndex > 0) this.selectedIndex--;
	}

	moveDown(): void {
		if (this.selectedIndex < this.filteredNotes.length - 1) this.selectedIndex++;
	}

	getSelected(): WikilinkSuggestion | null {
		return this.filteredNotes[this.selectedIndex] ?? null;
	}

	dismiss(): void {
		this.active = false;
		this.query = '';
		this.filteredNotes = [];
	}
}
