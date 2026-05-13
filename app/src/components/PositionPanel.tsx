"use client";

import React, { useEffect, useState } from "react";
import { useConnection, useWallet } from "@solana/wallet-adapter-react";
import { PublicKey, Transaction } from "@solana/web3.js";
import {
  fetchUserPosition,
  findUserPositionPda,
  findVaultAuthorityPda,
  redeemInstruction,
  PROGRAM_ID,
} from "@polymarket-sol/sdk";
import type { UserPosition, Market } from "@polymarket-sol/sdk";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

interface Props {
  marketPubkey: PublicKey;
  market:       Market;
}

function fmt(units: bigint): string {
  return (Number(units) / 1_000_000).toFixed(2);
}

export function PositionPanel({ marketPubkey, market }: Props) {
  const { connection }               = useConnection();
  const { publicKey, sendTransaction } = useWallet();
  const [pos,   setPos]              = useState<UserPosition | null>(null);
  const [busy,  setBusy]             = useState(false);
  const [error, setError]            = useState<string | null>(null);

  useEffect(() => {
    if (!publicKey) { setPos(null); return; }
    const [posPda] = findUserPositionPda(marketPubkey, publicKey, PROGRAM_ID);
    fetchUserPosition(connection, posPda)
      .then(setPos)
      .catch(() => setPos(null));
  }, [connection, publicKey, marketPubkey]);

  async function redeem(amount: bigint) {
    if (!publicKey || !pos) return;
    setBusy(true);
    setError(null);
    try {
      const [posPda]        = findUserPositionPda(marketPubkey, publicKey, PROGRAM_ID);
      const [vaultAuthPda]  = findVaultAuthorityPda(marketPubkey, PROGRAM_ID);
      const userUsdcAta     = getAssociatedTokenAddressSync(market.collateralMint, publicKey);

      const ix = redeemInstruction(
        publicKey,
        marketPubkey,
        posPda,
        userUsdcAta,
        market.vault,
        vaultAuthPda,
        amount,
        PROGRAM_ID,
      );
      const tx  = new Transaction().add(ix);
      const sig = await sendTransaction(tx, connection);
      await connection.confirmTransaction(sig, "confirmed");

      // Refresh position
      fetchUserPosition(connection, posPda).then(setPos).catch(() => {});
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  if (!publicKey) {
    return (
      <div className="bg-panel rounded-xl border border-border p-4 text-sm text-slate-400">
        Connect wallet to view positions.
      </div>
    );
  }

  if (!pos) {
    return (
      <div className="bg-panel rounded-xl border border-border p-4 text-sm text-slate-400">
        No position in this market.
      </div>
    );
  }

  const freeYes = pos.yesBalance - pos.lockedYes;
  const freeNo  = pos.noBalance  - pos.lockedNo;
  const isResolved = market.resolved;
  const canRedeem  = isResolved && (market.winningOutcome === 1 ? freeYes > 0n : freeNo > 0n);

  return (
    <div className="bg-panel rounded-xl border border-border p-4 space-y-3">
      <h3 className="text-sm font-semibold text-white">Your Position</h3>

      <div className="grid grid-cols-2 gap-2 text-sm">
        <Stat label="YES balance" value={`${fmt(pos.yesBalance)} USDC`} color="yes" />
        <Stat label="NO balance"  value={`${fmt(pos.noBalance)} USDC`}  color="no" />
        <Stat label="Locked YES"  value={fmt(pos.lockedYes)}  />
        <Stat label="Locked NO"   value={fmt(pos.lockedNo)}   />
        <Stat label="Locked collateral" value={fmt(pos.lockedCollateral)} />
        <Stat label="Open orders" value={String(pos.openOrderCount)} />
      </div>

      {isResolved && (
        <div className="rounded-lg bg-surface border border-border px-3 py-2 text-xs">
          Market resolved:{" "}
          <span className={market.winningOutcome === 1 ? "text-yes font-bold" : "text-no font-bold"}>
            {market.winningOutcome === 1 ? "YES" : "NO"} wins
          </span>
        </div>
      )}

      {canRedeem && (
        <button
          disabled={busy}
          onClick={() =>
            redeem(market.winningOutcome === 1 ? freeYes : freeNo)
          }
          className="w-full py-2.5 rounded-lg text-sm font-semibold bg-yes text-white hover:bg-yes-muted transition-colors disabled:opacity-40"
        >
          {busy ? "Redeeming…" : "Redeem Winnings"}
        </button>
      )}

      {error && <p className="text-xs text-red-400">{error}</p>}
    </div>
  );
}

function Stat({
  label,
  value,
  color,
}: {
  label: string;
  value: string;
  color?: "yes" | "no";
}) {
  const cls = color === "yes" ? "text-yes" : color === "no" ? "text-no" : "text-white";
  return (
    <div className="bg-surface rounded-lg px-2 py-1.5">
      <p className="text-xs text-slate-400">{label}</p>
      <p className={`text-sm font-mono font-semibold ${cls}`}>{value}</p>
    </div>
  );
}
