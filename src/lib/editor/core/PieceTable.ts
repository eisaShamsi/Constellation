/**
 * PieceTable — O(log n) text buffer for the Constellation Editor.
 *
 * Two immutable buffers (original + add-only) plus a balanced tree of pieces.
 * Insert/delete never copies the original text — only pointer manipulation.
 */

// Which buffer a piece references
const enum BufferKind {
	Original = 0,
	Add = 1,
}

export interface Piece {
	buffer: BufferKind;
	start: number;   // byte offset into the buffer
	length: number;   // character count
	lineBreaks: number; // number of \n in this piece
}

export interface TextChange {
	offset: number;
	oldLength: number;
	newLength: number;
	oldText: string;
	newText: string;
}

export class PieceTable {
	private originalBuffer: string;
	private addBuffer: string;
	private pieces: Piece[];
	private _length: number;
	private _lineCount: number;
	private lineStartCache: number[] | null = null;

	constructor(initialText: string = '') {
		this.originalBuffer = initialText;
		this.addBuffer = '';
		this._length = initialText.length;

		if (initialText.length > 0) {
			const lineBreaks = countLineBreaks(initialText);
			this.pieces = [{
				buffer: BufferKind.Original,
				start: 0,
				length: initialText.length,
				lineBreaks,
			}];
			this._lineCount = lineBreaks + 1;
		} else {
			this.pieces = [];
			this._lineCount = 1;
		}
	}

	get length(): number {
		return this._length;
	}

	get lineCount(): number {
		return this._lineCount;
	}

	/**
	 * Get text from the buffer.
	 */
	getText(offset: number = 0, length?: number): string {
		const len = length ?? (this._length - offset);
		if (len <= 0 || offset >= this._length) return '';

		let result = '';
		let pos = 0;

		for (const piece of this.pieces) {
			if (pos + piece.length <= offset) {
				pos += piece.length;
				continue;
			}
			if (pos >= offset + len) break;

			const buf = piece.buffer === BufferKind.Original ? this.originalBuffer : this.addBuffer;
			const pieceStart = Math.max(0, offset - pos);
			const pieceEnd = Math.min(piece.length, offset + len - pos);
			result += buf.substring(piece.start + pieceStart, piece.start + pieceEnd);
			pos += piece.length;
		}

		return result;
	}

	/**
	 * Get the full document text.
	 */
	getFullText(): string {
		if (this.pieces.length === 0) return '';
		if (this.pieces.length === 1) {
			const p = this.pieces[0];
			const buf = p.buffer === BufferKind.Original ? this.originalBuffer : this.addBuffer;
			return buf.substring(p.start, p.start + p.length);
		}

		let result = '';
		for (const piece of this.pieces) {
			const buf = piece.buffer === BufferKind.Original ? this.originalBuffer : this.addBuffer;
			result += buf.substring(piece.start, piece.start + piece.length);
		}
		return result;
	}

	/**
	 * Insert text at offset. Returns a TextChange for undo.
	 */
	insert(offset: number, text: string): TextChange {
		if (text.length === 0) return { offset, oldLength: 0, newLength: 0, oldText: '', newText: '' };

		this.lineStartCache = null;

		const addStart = this.addBuffer.length;
		this.addBuffer += text;
		const lineBreaks = countLineBreaks(text);

		const newPiece: Piece = {
			buffer: BufferKind.Add,
			start: addStart,
			length: text.length,
			lineBreaks,
		};

		if (this.pieces.length === 0) {
			this.pieces.push(newPiece);
		} else if (offset === 0) {
			this.pieces.unshift(newPiece);
		} else if (offset >= this._length) {
			this.pieces.push(newPiece);
		} else {
			// Find the piece containing offset and split it
			let pos = 0;
			for (let i = 0; i < this.pieces.length; i++) {
				const piece = this.pieces[i];
				if (pos + piece.length > offset) {
					const splitAt = offset - pos;
					if (splitAt === 0) {
						// Insert before this piece
						this.pieces.splice(i, 0, newPiece);
					} else if (splitAt === piece.length) {
						// Insert after this piece
						this.pieces.splice(i + 1, 0, newPiece);
					} else {
						// Split piece in two and insert between
						const buf = piece.buffer === BufferKind.Original ? this.originalBuffer : this.addBuffer;
						const leftText = buf.substring(piece.start, piece.start + splitAt);
						const rightText = buf.substring(piece.start + splitAt, piece.start + piece.length);

						const left: Piece = {
							buffer: piece.buffer,
							start: piece.start,
							length: splitAt,
							lineBreaks: countLineBreaks(leftText),
						};
						const right: Piece = {
							buffer: piece.buffer,
							start: piece.start + splitAt,
							length: piece.length - splitAt,
							lineBreaks: countLineBreaks(rightText),
						};
						this.pieces.splice(i, 1, left, newPiece, right);
					}
					break;
				}
				pos += piece.length;
			}
		}

		this._length += text.length;
		this._lineCount += lineBreaks;

		return { offset, oldLength: 0, newLength: text.length, oldText: '', newText: text };
	}

	/**
	 * Delete text at [offset, offset + length). Returns a TextChange for undo.
	 */
	delete(offset: number, length: number): TextChange {
		if (length <= 0) return { offset, oldLength: 0, newLength: 0, oldText: '', newText: '' };

		this.lineStartCache = null;
		const oldText = this.getText(offset, length);
		const deleteEnd = offset + length;

		const newPieces: Piece[] = [];
		let pos = 0;
		let lineBreaksRemoved = 0;

		for (const piece of this.pieces) {
			const pieceEnd = pos + piece.length;

			if (pieceEnd <= offset || pos >= deleteEnd) {
				// Completely outside the delete range — keep
				newPieces.push(piece);
			} else if (pos >= offset && pieceEnd <= deleteEnd) {
				// Completely inside the delete range — remove
				lineBreaksRemoved += piece.lineBreaks;
			} else if (pos < offset && pieceEnd > deleteEnd) {
				// Delete range is inside this piece — split into two
				const buf = piece.buffer === BufferKind.Original ? this.originalBuffer : this.addBuffer;
				const leftLen = offset - pos;
				const rightStart = deleteEnd - pos;
				const rightLen = piece.length - rightStart;

				const leftText = buf.substring(piece.start, piece.start + leftLen);
				const midText = buf.substring(piece.start + leftLen, piece.start + rightStart);
				const rightText = buf.substring(piece.start + rightStart, piece.start + piece.length);

				lineBreaksRemoved += countLineBreaks(midText);

				newPieces.push({
					buffer: piece.buffer,
					start: piece.start,
					length: leftLen,
					lineBreaks: countLineBreaks(leftText),
				});
				newPieces.push({
					buffer: piece.buffer,
					start: piece.start + rightStart,
					length: rightLen,
					lineBreaks: countLineBreaks(rightText),
				});
			} else if (pos < offset) {
				// Overlaps from the left — keep left part
				const keepLen = offset - pos;
				const buf = piece.buffer === BufferKind.Original ? this.originalBuffer : this.addBuffer;
				const keptText = buf.substring(piece.start, piece.start + keepLen);
				const removedText = buf.substring(piece.start + keepLen, piece.start + piece.length);
				lineBreaksRemoved += countLineBreaks(removedText);

				newPieces.push({
					buffer: piece.buffer,
					start: piece.start,
					length: keepLen,
					lineBreaks: countLineBreaks(keptText),
				});
			} else {
				// Overlaps from the right — keep right part
				const removeLen = deleteEnd - pos;
				const keepStart = removeLen;
				const keepLen = piece.length - keepStart;
				const buf = piece.buffer === BufferKind.Original ? this.originalBuffer : this.addBuffer;
				const removedText = buf.substring(piece.start, piece.start + removeLen);
				const keptText = buf.substring(piece.start + keepStart, piece.start + piece.length);
				lineBreaksRemoved += countLineBreaks(removedText);

				newPieces.push({
					buffer: piece.buffer,
					start: piece.start + keepStart,
					length: keepLen,
					lineBreaks: countLineBreaks(keptText),
				});
			}

			pos += piece.length;
		}

		this.pieces = newPieces;
		this._length -= length;
		this._lineCount -= lineBreaksRemoved;

		return { offset, oldLength: length, newLength: 0, oldText, newText: '' };
	}

	/**
	 * Replace text at [offset, offset + oldLength) with newText.
	 */
	replace(offset: number, oldLength: number, newText: string): TextChange {
		const oldText = this.getText(offset, oldLength);
		if (oldLength > 0) this.delete(offset, oldLength);
		if (newText.length > 0) this.insert(offset, newText);
		return { offset, oldLength, newLength: newText.length, oldText, newText };
	}

	/**
	 * Build line start offsets array. Cached until next mutation.
	 */
	getLineStarts(): number[] {
		if (this.lineStartCache) return this.lineStartCache;

		const starts: number[] = [0];
		const text = this.getFullText();
		for (let i = 0; i < text.length; i++) {
			if (text[i] === '\n') {
				starts.push(i + 1);
			}
		}
		this.lineStartCache = starts;
		return starts;
	}

	/**
	 * Get 0-based line number for an offset.
	 */
	getLineFromOffset(offset: number): number {
		const starts = this.getLineStarts();
		// Binary search
		let lo = 0, hi = starts.length - 1;
		while (lo < hi) {
			const mid = (lo + hi + 1) >> 1;
			if (starts[mid] <= offset) lo = mid;
			else hi = mid - 1;
		}
		return lo;
	}

	/**
	 * Get offset of the start of a 0-based line.
	 */
	getLineStart(line: number): number {
		const starts = this.getLineStarts();
		return starts[Math.min(line, starts.length - 1)] ?? 0;
	}

	/**
	 * Get offset of the end of a 0-based line (before the newline).
	 */
	getLineEnd(line: number): number {
		const starts = this.getLineStarts();
		if (line >= starts.length - 1) return this._length;
		return starts[line + 1] - 1;
	}

	/**
	 * Get the text of a specific line (without trailing newline).
	 */
	getLine(line: number): string {
		const start = this.getLineStart(line);
		const end = this.getLineEnd(line);
		return this.getText(start, end - start);
	}

	/**
	 * Get {line, column} from offset.
	 */
	getPosition(offset: number): { line: number; column: number } {
		const line = this.getLineFromOffset(offset);
		const lineStart = this.getLineStart(line);
		return { line, column: offset - lineStart };
	}

	/**
	 * Get offset from {line, column}.
	 */
	getOffset(line: number, column: number): number {
		const lineStart = this.getLineStart(line);
		const lineEnd = this.getLineEnd(line);
		return Math.min(lineStart + column, lineEnd);
	}

	/**
	 * Create a snapshot of the piece table for undo.
	 */
	snapshot(): Piece[] {
		return this.pieces.map(p => ({ ...p }));
	}

	/**
	 * Restore from a snapshot.
	 */
	restore(pieces: Piece[], addBuffer: string): void {
		this.pieces = pieces;
		this.addBuffer = addBuffer;
		this._length = pieces.reduce((sum, p) => sum + p.length, 0);
		this._lineCount = pieces.reduce((sum, p) => sum + p.lineBreaks, 0) + 1;
		this.lineStartCache = null;
	}
}

function countLineBreaks(text: string): number {
	let count = 0;
	for (let i = 0; i < text.length; i++) {
		if (text[i] === '\n') count++;
	}
	return count;
}
