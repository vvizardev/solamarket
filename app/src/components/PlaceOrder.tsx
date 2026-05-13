"use client";

import React, { useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { PublicKey, Transaction } from "@solana/web3.js";
import {
  placeOrderInstruction,
  findOrderPda,
  findUserPositionPda,
  OrderSide,
  PROGRAM_ID,
} from "@polymarket-sol/sdk";

interface Props {
  marketPubkey: PublicKey;
}

export function PlaceOrder({ marketPubkey }: Props) {
  const { connection }          = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const [side,  setSide]        = useState<OrderSide>(OrderSide.Bid);
  const [price, setPrice]       = useState("5000");   // in bps
  const [size,  setSize]        = useState("10");      // in USDC
  const [nonce, setNonce]       = useState(() => BigInt(Date.now()));
  const [busy,  setBusy]        = useState(false);
  const [error, setError]       = useState<string | null>(null);
  const [txSig, setTxSig]       = useState<string | null>(null);

  async function submit(e: React.FormEvent) {
    e.preventDefault();
    if (!publicKey) return;
    setBusy(true);
    setError(null);
    setTxSig(null);

    try {
      const priceBps  = BigInt(price);
      const sizeUnits = BigInt(Math.round(parseFloat(size) * 1_000_000));
      const currentNonce = nonce;

      const [userPosPda] = findUserPositionPda(marketPubkey, publicKey, PROGRAM_ID);
      const [orderPda]   = findOrderPda(marketPubkey, publicKey, currentNonce, PROGRAM_ID);

      const ix = placeOrderInstruction(
        publicKey,
        marketPubkey,
        userPosPda,
        orderPda,
        { side, price: priceBps, size: sizeUnits, nonce: currentNonce },
        PROGRAM_ID,
      );

      const tx  = new Transaction().add(ix);
      const sig = await sendTransaction(tx, connection, { skipPreflight: false });
      await connection.confirmTransaction(sig, "confirmed");

      setTxSig(sig);
      setNonce((n) => n + 1n); // increment nonce for next order
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="bg-panel rounded-xl border border-border p-4">
      <h3 className="text-sm font-semibold text-white mb-4">Place Order</h3>

      {/* Side toggle */}
      <div className="flex rounded-lg overflow-hidden border border-border mb-4">
        <button
          className={`flex-1 py-2 text-sm font-semibold transition-colors ${
            side === OrderSide.Bid
              ? "bg-yes text-white"
              : "text-slate-400 hover:bg-white/5"
          }`}
          onClick={() => setSide(OrderSide.Bid)}
          type="button"
        >
          Buy YES
        </button>
        <button
          className={`flex-1 py-2 text-sm font-semibold transition-colors ${
            side === OrderSide.Ask
              ? "bg-no text-white"
              : "text-slate-400 hover:bg-white/5"
          }`}
          onClick={() => setSide(OrderSide.Ask)}
          type="button"
        >
          Sell YES
        </button>
      </div>

      <form onSubmit={submit} className="space-y-3">
        <div>
          <label className="block text-xs text-slate-400 mb-1">
            Price (basis points, 1–9999)
          </label>
          <input
            type="number"
            min="1"
            max="9999"
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            className="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm text-white font-mono focus:outline-none focus:ring-1 focus:ring-yes"
          />
          <p className="text-xs text-slate-500 mt-1">
            = {(parseFloat(price) / 100).toFixed(2)}% implied probability
          </p>
        </div>

        <div>
          <label className="block text-xs text-slate-400 mb-1">Size (USDC)</label>
          <input
            type="number"
            min="0.01"
            step="0.01"
            value={size}
            onChange={(e) => setSize(e.target.value)}
            className="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm text-white font-mono focus:outline-none focus:ring-1 focus:ring-yes"
          />
        </div>

        {!publicKey && (
          <p className="text-xs text-slate-400">Connect wallet to trade.</p>
        )}

        <button
          type="submit"
          disabled={!publicKey || busy}
          className={`w-full py-2.5 rounded-lg text-sm font-semibold transition-colors disabled:opacity-40 ${
            side === OrderSide.Bid
              ? "bg-yes hover:bg-yes-muted text-white"
              : "bg-no  hover:bg-no-muted  text-white"
          }`}
        >
          {busy ? "Sending…" : side === OrderSide.Bid ? "Place Bid" : "Place Ask"}
        </button>
      </form>

      {error && (
        <p className="mt-3 text-xs text-red-400 break-all">{error}</p>
      )}
      {txSig && (
        <a
          href={`https://explorer.solana.com/tx/${txSig}?cluster=devnet`}
          target="_blank"
          rel="noopener noreferrer"
          className="mt-3 block text-xs text-yes underline break-all"
        >
          Confirmed: {txSig.slice(0, 20)}…
        </a>
      )}
    </div>
  );
}
