import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { useCallback, useEffect, useState } from "react";
import { deserializeMarket, PROGRAM_ID } from "@polymarket-sol/sdk";
import type { Market } from "@polymarket-sol/sdk";

export interface MarketWithPubkey {
  pubkey: PublicKey;
  market: Market;
}

export function useMarkets(): {
  markets: MarketWithPubkey[];
  loading: boolean;
  error:   string | null;
  refresh: () => void;
} {
  const { connection }              = useConnection();
  const [markets, setMarkets]       = useState<MarketWithPubkey[]>([]);
  const [loading, setLoading]       = useState(false);
  const [error,   setError]         = useState<string | null>(null);

  const fetch = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const accounts = await connection.getProgramAccounts(PROGRAM_ID, {
        filters: [{ memcmp: { offset: 0, bytes: Buffer.from([0]).toString("base64") } }],
      });
      const parsed = accounts
        .map(({ pubkey, account }) => {
          try {
            return { pubkey, market: deserializeMarket(Buffer.from(account.data)) };
          } catch {
            return null;
          }
        })
        .filter((x): x is MarketWithPubkey => x !== null);
      setMarkets(parsed);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [connection]);

  useEffect(() => { fetch(); }, [fetch]);

  return { markets, loading, error, refresh: fetch };
}
