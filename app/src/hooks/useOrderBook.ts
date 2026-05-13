import { useConnection } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { useEffect, useRef, useState } from "react";
import { OrderSubscriber, DLOB, DLOBNode, PROGRAM_ID } from "@polymarket-sol/sdk";

export interface OrderBookState {
  bids:    DLOBNode[];
  asks:    DLOBNode[];
  spread:  bigint | null;
}

/**
 * Subscribes to the live order book for a given market.
 * Cleans up the WebSocket subscription on unmount.
 */
export function useOrderBook(marketPubkey: PublicKey | null): OrderBookState {
  const { connection }                            = useConnection();
  const subscriberRef                             = useRef<OrderSubscriber | null>(null);
  const [bids, setBids]                           = useState<DLOBNode[]>([]);
  const [asks, setAsks]                           = useState<DLOBNode[]>([]);

  useEffect(() => {
    if (!marketPubkey) return;

    const subscriber = new OrderSubscriber(connection, marketPubkey, PROGRAM_ID);
    subscriberRef.current = subscriber;

    function syncState(): void {
      const dlob = subscriber.getDLOB();
      setBids(dlob.sortedBids().slice(0, 10));
      setAsks(dlob.sortedAsks().slice(0, 10));
    }

    subscriber.onUpdate(() => syncState());

    subscriber.subscribe().then(syncState).catch(console.error);

    return () => {
      subscriber.unsubscribe();
      subscriberRef.current = null;
    };
  }, [connection, marketPubkey?.toBase58()]);

  const spread: bigint | null =
    bids[0] && asks[0] ? asks[0].price - bids[0].price : null;

  return { bids, asks, spread };
}
