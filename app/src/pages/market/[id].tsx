import type { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import Link from "next/link";
import { PublicKey } from "@solana/web3.js";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useConnection } from "@solana/wallet-adapter-react";
import { useEffect, useState } from "react";
import { fetchMarket, PROGRAM_ID } from "@polymarket-sol/sdk";
import type { Market } from "@polymarket-sol/sdk";
import { useOrderBook } from "@/hooks/useOrderBook";
import { OrderBook } from "@/components/OrderBook";
import { PlaceOrder } from "@/components/PlaceOrder";
import { PositionPanel } from "@/components/PositionPanel";
import React from "react";

const MarketDetail: NextPage = () => {
  const router = useRouter();
  const id     = typeof router.query.id === "string" ? router.query.id : null;

  const { connection }                = useConnection();
  const [market, setMarket]           = useState<Market | null>(null);
  const [marketPk, setMarketPk]       = useState<PublicKey | null>(null);
  const [loadErr, setLoadErr]         = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    let pk: PublicKey;
    try {
      pk = new PublicKey(id);
    } catch {
      setLoadErr("Invalid market address");
      return;
    }
    setMarketPk(pk);
    fetchMarket(connection, pk)
      .then(setMarket)
      .catch((e) => setLoadErr(e.message));
  }, [connection, id]);

  const { bids, asks, spread } = useOrderBook(marketPk);

  if (loadErr) {
    return (
      <div className="min-h-screen bg-surface flex items-center justify-center text-red-400">
        {loadErr}
      </div>
    );
  }

  if (!market || !marketPk) {
    return (
      <div className="min-h-screen bg-surface flex items-center justify-center text-slate-400">
        Loading…
      </div>
    );
  }

  const question = `0x${Buffer.from(market.questionHash).toString("hex")}`;

  return (
    <>
      <Head>
        <title>Market — Prediction Markets</title>
      </Head>

      <div className="min-h-screen bg-surface">
        {/* Nav */}
        <nav className="border-b border-border">
          <div className="max-w-6xl mx-auto px-4 h-14 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Link href="/" className="text-slate-400 hover:text-white text-sm">
                ← Markets
              </Link>
              <span className="text-slate-600">/</span>
              <span className="text-sm font-mono text-white">
                {marketPk.toBase58().slice(0, 12)}…
              </span>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-xs text-slate-400 bg-slate-800 px-2 py-1 rounded-md">
                Devnet
              </span>
              <WalletMultiButton className="!bg-yes !text-white !text-sm !py-1.5 !px-4 !rounded-lg" />
            </div>
          </div>
        </nav>

        {/* Header */}
        <div className="border-b border-border bg-panel">
          <div className="max-w-6xl mx-auto px-4 py-4">
            <p className="text-xs text-slate-400 font-mono mb-1 break-all">{question}</p>
            <div className="flex items-center gap-3">
              <span
                className={`text-xs px-2 py-0.5 rounded-full font-medium ${
                  market.resolved
                    ? "bg-slate-700 text-slate-300"
                    : "bg-yes/20 text-yes"
                }`}
              >
                {market.resolved ? `Resolved: ${market.winningOutcome === 1 ? "YES" : "NO"}` : "Live"}
              </span>
              <span className="text-xs text-slate-400">
                Ends {new Date(Number(market.endTime) * 1000).toLocaleString()}
              </span>
            </div>
          </div>
        </div>

        {/* Main 3-column layout */}
        <main className="max-w-6xl mx-auto px-4 py-6 grid gap-4 lg:grid-cols-[1fr_300px_300px]">
          {/* Order book */}
          <OrderBook bids={bids} asks={asks} spread={spread} />

          {/* Place order */}
          <PlaceOrder marketPubkey={marketPk} />

          {/* Position */}
          <PositionPanel marketPubkey={marketPk} market={market} />
        </main>
      </div>
    </>
  );
};

export default MarketDetail;
