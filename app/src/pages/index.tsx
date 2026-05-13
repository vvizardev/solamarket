import type { NextPage } from "next";
import Head from "next/head";
import Link from "next/link";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useMarkets } from "@/hooks/useMarkets";
import React from "react";

function StatusBadge({ resolved }: { resolved: boolean }) {
  return (
    <span
      className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${
        resolved
          ? "bg-slate-700 text-slate-300"
          : "bg-yes/20 text-yes"
      }`}
    >
      <span className={`w-1.5 h-1.5 rounded-full ${resolved ? "bg-slate-400" : "bg-yes"}`} />
      {resolved ? "Resolved" : "Live"}
    </span>
  );
}

const Home: NextPage = () => {
  const { markets, loading, error, refresh } = useMarkets();

  return (
    <>
      <Head>
        <title>Prediction Markets — Solana Devnet</title>
        <meta name="description" content="Polymarket-style prediction markets on Solana using a DLOB" />
      </Head>

      <div className="min-h-screen bg-surface">
        {/* Nav */}
        <nav className="border-b border-border">
          <div className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
            <span className="font-bold text-white tracking-tight">
              🔮 Prediction Markets
            </span>
            <div className="flex items-center gap-3">
              <span className="text-xs text-slate-400 bg-slate-800 px-2 py-1 rounded-md">
                Devnet
              </span>
              <WalletMultiButton className="!bg-yes !text-white !text-sm !py-1.5 !px-4 !rounded-lg" />
            </div>
          </div>
        </nav>

        {/* Content */}
        <main className="max-w-6xl mx-auto px-4 py-8">
          <div className="flex items-center justify-between mb-6">
            <h1 className="text-2xl font-bold text-white">Open Markets</h1>
            <button
              onClick={refresh}
              disabled={loading}
              className="text-sm text-slate-400 hover:text-white transition-colors"
            >
              {loading ? "Loading…" : "↻ Refresh"}
            </button>
          </div>

          {error && (
            <div className="mb-4 p-3 bg-red-900/30 border border-red-800 rounded-lg text-sm text-red-300">
              {error}
            </div>
          )}

          {markets.length === 0 && !loading && (
            <div className="text-center py-16 text-slate-500">
              <p className="text-lg">No markets found.</p>
              <p className="text-sm mt-1">
                Run <code className="text-slate-300">scripts/create-market.ts</code> to create one.
              </p>
            </div>
          )}

          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {markets.map(({ pubkey, market }) => {
              const question = `0x${Buffer.from(market.questionHash).toString("hex").slice(0, 16)}…`;
              return (
                <Link
                  key={pubkey.toBase58()}
                  href={`/market/${pubkey.toBase58()}`}
                  className="block bg-panel border border-border rounded-xl p-4 hover:border-slate-400 transition-colors group"
                >
                  <div className="flex items-start justify-between mb-3">
                    <p className="text-xs text-slate-400 font-mono">{question}</p>
                    <StatusBadge resolved={market.resolved} />
                  </div>
                  <p className="text-sm text-slate-300 font-mono mb-4 truncate">
                    {pubkey.toBase58().slice(0, 20)}…
                  </p>
                  <div className="flex items-center justify-between text-xs text-slate-400">
                    <span>Ends {new Date(Number(market.endTime) * 1000).toLocaleDateString()}</span>
                    {market.resolved && (
                      <span className={market.winningOutcome === 1 ? "text-yes font-semibold" : "text-no font-semibold"}>
                        {market.winningOutcome === 1 ? "YES wins" : "NO wins"}
                      </span>
                    )}
                  </div>
                </Link>
              );
            })}
          </div>
        </main>
      </div>
    </>
  );
};

export default Home;
