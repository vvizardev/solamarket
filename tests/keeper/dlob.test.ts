import { PublicKey } from "@solana/web3.js";
import { DLOB } from "../../sdk/src/dlob/DLOB";
import { DLOBNode } from "../../sdk/src/dlob/DLOBNode";
import type { Order } from "../../sdk/src/types";
import { OrderSide } from "../../sdk/src/types";

// ── helpers ──────────────────────────────────────────────────────────────────

function makePubkey(): PublicKey {
  return PublicKey.unique();
}

function makeOrder(overrides: Partial<Order> = {}): Order {
  return {
    discriminant: 1,
    market:       makePubkey(),
    user:         makePubkey(),
    side:         OrderSide.Bid,
    price:        5000n,
    size:         1_000_000n,
    fillAmount:   0n,
    nonce:        0n,
    createdAt:    BigInt(Date.now()),
    bump:         255,
    ...overrides,
  };
}

function makeNode(price: bigint, side: OrderSide, fillAmount = 0n): DLOBNode {
  const order = makeOrder({ price, side, fillAmount });
  return new DLOBNode(makePubkey(), order);
}

// ── DLOB unit tests ───────────────────────────────────────────────────────────

describe("DLOB", () => {
  it("inserts bids and asks correctly", () => {
    const dlob = new DLOB();
    dlob.insert(makeNode(5000n, OrderSide.Bid));
    dlob.insert(makeNode(4000n, OrderSide.Bid));
    dlob.insert(makeNode(6000n, OrderSide.Ask));

    expect(dlob.bidCount).toBe(2);
    expect(dlob.askCount).toBe(1);
  });

  it("bestBid returns highest price bid", () => {
    const dlob = new DLOB();
    dlob.insert(makeNode(4000n, OrderSide.Bid));
    dlob.insert(makeNode(5500n, OrderSide.Bid));
    dlob.insert(makeNode(3000n, OrderSide.Bid));

    expect(dlob.bestBid()?.price).toBe(5500n);
  });

  it("bestAsk returns lowest price ask", () => {
    const dlob = new DLOB();
    dlob.insert(makeNode(7000n, OrderSide.Ask));
    dlob.insert(makeNode(5500n, OrderSide.Ask));
    dlob.insert(makeNode(9000n, OrderSide.Ask));

    expect(dlob.bestAsk()?.price).toBe(5500n);
  });

  it("findCross returns pair when bid >= ask", () => {
    const dlob = new DLOB();
    dlob.insert(makeNode(6000n, OrderSide.Bid));
    dlob.insert(makeNode(5500n, OrderSide.Ask));

    const cross = dlob.findCross();
    expect(cross).not.toBeUndefined();
    expect(cross![0].price).toBe(6000n); // bid
    expect(cross![1].price).toBe(5500n); // ask
  });

  it("findCross returns undefined when no crossing", () => {
    const dlob = new DLOB();
    dlob.insert(makeNode(4000n, OrderSide.Bid));
    dlob.insert(makeNode(6000n, OrderSide.Ask));

    expect(dlob.findCross()).toBeUndefined();
  });

  it("removes a fully-filled order on update", () => {
    const dlob = new DLOB();
    const node = makeNode(5000n, OrderSide.Bid);
    dlob.insert(node);
    expect(dlob.bidCount).toBe(1);

    const filledNode = new DLOBNode(node.pubkey, { ...node.order, fillAmount: node.order.size });
    dlob.update(node.pubkey, filledNode);
    expect(dlob.bidCount).toBe(0);
  });

  it("sortedBids are price DESC, FIFO for equal prices", () => {
    const dlob = new DLOB();
    const o1   = makeOrder({ price: 5000n, side: OrderSide.Bid, createdAt: 100n });
    const o2   = makeOrder({ price: 5000n, side: OrderSide.Bid, createdAt: 200n });
    const o3   = makeOrder({ price: 6000n, side: OrderSide.Bid, createdAt: 50n  });

    dlob.insert(new DLOBNode(makePubkey(), o1));
    dlob.insert(new DLOBNode(makePubkey(), o2));
    dlob.insert(new DLOBNode(makePubkey(), o3));

    const bids = dlob.sortedBids();
    expect(bids[0].price).toBe(6000n);
    expect(bids[1].order.createdAt).toBe(100n); // FIFO: o1 before o2
    expect(bids[2].order.createdAt).toBe(200n);
  });
});

// ── DLOBNode unit tests ───────────────────────────────────────────────────────

describe("DLOBNode", () => {
  it("computes remaining correctly", () => {
    const order = makeOrder({ size: 1_000_000n, fillAmount: 400_000n });
    const node  = new DLOBNode(makePubkey(), order);
    expect(node.remaining).toBe(600_000n);
  });

  it("isFullyFilled when fillAmount >= size", () => {
    const order = makeOrder({ size: 1_000_000n, fillAmount: 1_000_000n });
    const node  = new DLOBNode(makePubkey(), order);
    expect(node.isFullyFilled).toBe(true);
  });

  it("applyFill updates cached fill", () => {
    const order = makeOrder({ size: 1_000_000n, fillAmount: 0n });
    const node  = new DLOBNode(makePubkey(), order);
    node.applyFill(300_000n);
    expect(node.remaining).toBe(700_000n);
  });

  it("reconcile only advances fill amount, never decreases", () => {
    const order = makeOrder({ size: 1_000_000n, fillAmount: 200_000n });
    const node  = new DLOBNode(makePubkey(), order);
    node.applyFill(300_000n); // cached = 500_000

    node.reconcile(100_000n); // stale — should not decrease
    expect(node.remaining).toBe(500_000n);

    node.reconcile(800_000n); // fresher — should advance
    expect(node.remaining).toBe(200_000n);
  });
});
