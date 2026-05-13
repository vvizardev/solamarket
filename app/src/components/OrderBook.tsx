import React from "react";
import { DLOBNode } from "@polymarket-sol/sdk";

interface Props {
  bids:   DLOBNode[];
  asks:   DLOBNode[];
  spread: bigint | null;
}

function formatPrice(bps: bigint): string {
  return `${(Number(bps) / 100).toFixed(2)}¢`;
}

function formatSize(units: bigint): string {
  return (Number(units) / 1_000_000).toFixed(2);
}

interface RowProps {
  node:  DLOBNode;
  side:  "bid" | "ask";
  depth: number; // 0–1 fraction for bar width
}

function OrderRow({ node, side, depth }: RowProps) {
  const color = side === "bid" ? "bg-yes/20" : "bg-no/20";
  const text  = side === "bid" ? "text-yes"  : "text-no";
  return (
    <div className="relative flex items-center px-3 py-1 text-xs font-mono hover:bg-white/5 transition-colors">
      {/* depth bar */}
      <div
        className={`absolute inset-y-0 ${side === "bid" ? "right-0" : "left-0"} ${color}`}
        style={{ width: `${(depth * 100).toFixed(1)}%` }}
      />
      <span className={`flex-1 ${text} font-semibold z-10`}>
        {formatPrice(node.price)}
      </span>
      <span className="text-slate-300 z-10">{formatSize(node.remaining)}</span>
    </div>
  );
}

export function OrderBook({ bids, asks, spread }: Props) {
  const maxSize = [...bids, ...asks].reduce(
    (m, n) => (n.remaining > m ? n.remaining : m),
    1n,
  );

  return (
    <div className="bg-panel rounded-xl border border-border overflow-hidden">
      <div className="px-4 py-3 border-b border-border">
        <h3 className="text-sm font-semibold text-white">Order Book</h3>
        {spread !== null && (
          <p className="text-xs text-slate-400 mt-0.5">
            Spread: {formatPrice(spread)}
          </p>
        )}
      </div>

      {/* Asks (reversed so highest ask is at top of sell side) */}
      <div className="flex flex-col-reverse">
        {asks.map((n) => (
          <OrderRow
            key={n.pubkey.toBase58()}
            node={n}
            side="ask"
            depth={Number(n.remaining) / Number(maxSize)}
          />
        ))}
      </div>

      <div className="border-t border-b border-border px-3 py-1.5 text-center text-xs text-slate-500">
        ── spread ──
      </div>

      {/* Bids */}
      <div>
        {bids.map((n) => (
          <OrderRow
            key={n.pubkey.toBase58()}
            node={n}
            side="bid"
            depth={Number(n.remaining) / Number(maxSize)}
          />
        ))}
      </div>
    </div>
  );
}
