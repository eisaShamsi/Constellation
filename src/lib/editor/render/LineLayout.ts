/**
 * LineLayout — line measurement and height caching for virtual scrolling.
 */

export interface LineMeasurement {
	line: number;
	top: number;
	height: number;
	measured: boolean;
}

const DEFAULT_LINE_HEIGHT = 24;
const HEADING_HEIGHTS: Record<number, number> = {
	1: 48, 2: 40, 3: 34, 4: 30, 5: 28, 6: 26,
};

export class LineLayout {
	private measurements: LineMeasurement[] = [];
	private totalHeight = 0;
	private defaultHeight = DEFAULT_LINE_HEIGHT;

	constructor(lineCount: number) {
		this.rebuild(lineCount);
	}

	rebuild(lineCount: number): void {
		this.measurements = [];
		let top = 0;
		for (let i = 0; i < lineCount; i++) {
			const height = this.defaultHeight;
			this.measurements.push({ line: i, top, height, measured: false });
			top += height;
		}
		this.totalHeight = top;
	}

	/** Update the measured height of a line. */
	setLineHeight(line: number, height: number): void {
		if (line < 0 || line >= this.measurements.length) return;
		const m = this.measurements[line];
		if (m.height === height && m.measured) return;

		const diff = height - m.height;
		m.height = height;
		m.measured = true;

		// Update top positions for all subsequent lines
		for (let i = line + 1; i < this.measurements.length; i++) {
			this.measurements[i].top += diff;
		}
		this.totalHeight += diff;
	}

	/** Set estimated height for a heading line. */
	setHeadingHeight(line: number, level: number): void {
		const height = HEADING_HEIGHTS[level] ?? DEFAULT_LINE_HEIGHT;
		if (line < this.measurements.length && !this.measurements[line].measured) {
			const diff = height - this.measurements[line].height;
			this.measurements[line].height = height;
			for (let i = line + 1; i < this.measurements.length; i++) {
				this.measurements[i].top += diff;
			}
			this.totalHeight += diff;
		}
	}

	/** Get the visible line range for a viewport. */
	getVisibleRange(scrollTop: number, viewportHeight: number, buffer: number = 5): { start: number; end: number } {
		const start = Math.max(0, this.getLineAtY(scrollTop) - buffer);
		const end = Math.min(this.measurements.length, this.getLineAtY(scrollTop + viewportHeight) + buffer + 1);
		return { start, end };
	}

	/** Find the line at a given Y coordinate using binary search. */
	getLineAtY(y: number): number {
		if (this.measurements.length === 0) return 0;

		let lo = 0;
		let hi = this.measurements.length - 1;
		while (lo < hi) {
			const mid = (lo + hi + 1) >> 1;
			if (this.measurements[mid].top <= y) lo = mid;
			else hi = mid - 1;
		}
		return lo;
	}

	/** Get the Y position of a line. */
	getLineTop(line: number): number {
		if (line < 0) return 0;
		if (line >= this.measurements.length) return this.totalHeight;
		return this.measurements[line].top;
	}

	/** Get the height of a line. */
	getLineHeight(line: number): number {
		if (line < 0 || line >= this.measurements.length) return this.defaultHeight;
		return this.measurements[line].height;
	}

	getTotalHeight(): number {
		return this.totalHeight;
	}

	getLineCount(): number {
		return this.measurements.length;
	}

	/** Handle line count change after editing. */
	updateLineCount(newCount: number): void {
		if (newCount === this.measurements.length) return;

		if (newCount > this.measurements.length) {
			// Add new lines
			let top = this.totalHeight;
			for (let i = this.measurements.length; i < newCount; i++) {
				this.measurements.push({ line: i, top, height: this.defaultHeight, measured: false });
				top += this.defaultHeight;
			}
			this.totalHeight = top;
		} else {
			// Remove lines from end
			let removed = 0;
			for (let i = newCount; i < this.measurements.length; i++) {
				removed += this.measurements[i].height;
			}
			this.measurements.length = newCount;
			this.totalHeight -= removed;
		}
	}
}
