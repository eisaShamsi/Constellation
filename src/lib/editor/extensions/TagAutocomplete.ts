/**
 * TagAutocomplete — #tag suggestions for the Constellation Editor.
 *
 * Triggers when the user types # followed by characters.
 */

export class TagAutocomplete {
	private tags: string[] = [];
	private active = false;
	private query = '';
	private filteredTags: string[] = [];
	private selectedIndex = 0;

	setTags(tags: string[]): void {
		this.tags = tags;
	}

	checkTrigger(textBeforeCursor: string): boolean {
		// Look for # at the start of a word
		const match = textBeforeCursor.match(/(^|\s)#(\w*)$/);
		if (!match) {
			this.active = false;
			return false;
		}

		this.active = true;
		this.query = match[2];
		this.filter();
		return this.filteredTags.length > 0;
	}

	private filter(): void {
		const q = this.query.toLowerCase();
		this.filteredTags = this.tags
			.filter(t => t.toLowerCase().includes(q))
			.slice(0, 10);
		this.selectedIndex = 0;
	}

	isActive(): boolean {
		return this.active;
	}

	getResults(): string[] {
		return this.filteredTags;
	}

	getSelectedIndex(): number {
		return this.selectedIndex;
	}

	moveUp(): void {
		if (this.selectedIndex > 0) this.selectedIndex--;
	}

	moveDown(): void {
		if (this.selectedIndex < this.filteredTags.length - 1) this.selectedIndex++;
	}

	getSelected(): string | null {
		return this.filteredTags[this.selectedIndex] ?? null;
	}

	dismiss(): void {
		this.active = false;
		this.query = '';
		this.filteredTags = [];
	}
}
