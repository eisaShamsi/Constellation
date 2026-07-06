/**
 * Tag utilities — the shared DashboardTag shape for tag aggregation surfaces.
 * (The former scanAllLibraryTags fs-walk was retired by MIG-079 §C.1 / MIG-080 §B;
 * the Dashboard now reads the write-time tag_counts-derived allLibraryTags prop.)
 */

export interface DashboardTag {
	tag: string;
	count: number;
}
