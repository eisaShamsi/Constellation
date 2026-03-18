/**
 * GraphMind — Type definitions for the graph view engine.
 */

export interface GraphMindNode {
	id: string;
	name: string;
	path: string;
	libraryName: string;
	linkCount: number;
	outgoingCount: number;
	/** ISO date string of last modification (if available) */
	modified?: string;
}

export interface GraphMindEdge {
	source: string;
	target: string;
	/** Structural link type from wikilink/tag/etc */
	linkType?: string;
	/** Semantic relationship label (supports, contradicts, elaborates, questions, custom) */
	relationship?: string;
	/** Is this a semantic (AI-computed) link vs structural (explicit) link? */
	semantic?: boolean;
}

export interface GraphMindSettings {
	nodeSize: number;
	labelVisibility: 'hover' | 'always' | 'none';
	labelFontSize: number;
	linkThickness: number;
	repelForce: number;
	linkForce: number;
	linkDistance: number;
	showOrphans: boolean;
	colorByLibrary: boolean;
}

/** Message types for the force simulation Web Worker */
export type WorkerMessage =
	| { type: 'init'; nodes: WorkerNode[]; edges: WorkerEdge[]; settings: ForceSettings }
	| { type: 'updateSettings'; settings: ForceSettings }
	| { type: 'pinNode'; id: string; x: number; y: number }
	| { type: 'unpinNode'; id: string }
	| { type: 'stop' }
	| { type: 'restart' };

export type WorkerResponse =
	| { type: 'positions'; positions: Float64Array; settled: boolean }
	| { type: 'ready' };

export interface WorkerNode {
	id: string;
	x?: number;
	y?: number;
}

export interface WorkerEdge {
	source: string;
	target: string;
}

export interface ForceSettings {
	repelForce: number;
	linkForce: number;
	linkDistance: number;
	centerForce: number;
}
