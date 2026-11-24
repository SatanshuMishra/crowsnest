export interface Edges {
  top: number;
  bottom: number;
  left: number;
  right: number;
}

export interface Viewport {
  width: number;
  height: number;
  top: number;
  margin: number;
}

export interface Size {
  width: number;
  height: number;
}

export type Side = "below" | "above";

export interface Placement {
  side: Side;
  top: number | null;
  bottom: number | null;
  left: number;
  width: number;
  maxHeight: number;
}

function clamp(value: number, low: number, high: number): number {
  return Math.min(Math.max(value, low), Math.max(low, high));
}

export function placePopover(anchor: Edges, viewport: Viewport, size: Size): Placement {
  const width = Math.min(size.width, viewport.width - viewport.margin * 2);
  const left = clamp(anchor.right - width, viewport.margin, viewport.width - width - viewport.margin);
  const below = viewport.height - viewport.margin - anchor.bottom;
  const above = anchor.top - viewport.top - viewport.margin;
  const side: Side = size.height <= below || below >= above ? "below" : "above";
  return {
    side,
    top: side === "below" ? anchor.bottom : null,
    bottom: side === "above" ? viewport.height - anchor.top : null,
    left,
    width,
    maxHeight: Math.max(side === "below" ? below : above, 0),
  };
}
