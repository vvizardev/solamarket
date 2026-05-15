# Market Categories

> Polymarket-style navigation taxonomy stored compactly on-chain (`primary_category`, `subcategory`) on both `Market` and `Event` accounts.

---

## Why On-Chain Categories?

Storing **numeric category ids** (not question text) lets clients:

- Filter with `getProgramAccounts` **memcmp** at fixed offsets ([Program — Accounts](../program/accounts.md#getprogramaccounts-filters)).
- Apply **per-category fee caps** or risk rules without trusting an off-chain registry for the raw classification.
- Render browse UI consistently across indexers.

Human-readable labels and long-tail tags still live **off-chain** (JSON, CMS, IPFS); on-chain fields stay small for rent.

---

## Account Fields

| Account | `primary_category` | `subcategory` | Bump |
|---------|-------------------|---------------|------|
| `Market` | `u8` @ offset **291** | `u16` @ **292–293** | `u8` @ **294** |
| `Event` | `u8` @ offset **588** | `u16` @ **589–590** | `u8` @ **591** |

**Reserved values**

| Field | Value | Meaning |
|-------|------|---------|
| `primary_category` | `0` | **Uncategorized** (legacy or unset) |
| `subcategory` | `0` | **None** — no finer bucket; interpret as “whole primary only” |

Clients MUST treat unknown `primary_category` ids as **unknown / misc** until a registry mapping is updated.

The meaning of non-zero `subcategory` **depends on `primary_category`** — see tables below.

---

## Placement Rules

| Scenario | Where to set |
|----------|----------------|
| **Standalone market** (`market.event == default`) | Set `primary_category` + `subcategory` on the **`Market`** only. |
| **Multi-market event** | Set taxonomy on **`Event`** for the group (e.g. entire election = Politics). Each **`Market`** SHOULD duplicate the same `primary_category` / `subcategory` so filters can target markets **without** loading `Event`. Markets MAY use a **narrower** subcategory than the event (e.g. Event = Sports, Market = NFL). |

Convention: after `AddMarketToEvent`, keep child market categories **aligned** with the event unless there is a deliberate drill-down.

---

## Primary Categories (`primary_category`)

| Id | Name | Typical use |
|----|------|-------------|
| `0` | Uncategorized | Default / legacy |
| `1` | Politics | Elections, legislation, geopolitics |
| `2` | Sports | Leagues, games, awards |
| `3` | Crypto | Assets, protocols, regulation, short-horizon markets |
| `4` | Weather | Temperature, storms, air quality, geophysical |
| `5` | Economics | Rates, inflation, employment, GDP |
| `6` | Companies & tech | Products, earnings, M&A |
| `7` | Science & health | Research, trials, space |
| `8` | Culture & entertainment | Film, music, awards shows |
| `9` | World & disasters | Major incidents (may overlap Politics / Weather) |

Ids **`10`–`255`** are reserved for future registry expansion.

---

## Subcategories by Primary

When `subcategory != 0`, interpret it using the parent primary. Values not listed are reserved.

### Politics (`primary_category = 1`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | Elections |
| `2` | Legislation & policy |
| `3` | Geopolitics |

### Sports (`primary_category = 2`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | American football |
| `2` | Basketball |
| `3` | Soccer / football |
| `4` | Baseball |
| `5` | Combat sports |
| `6` | Motorsports |
| `7` | Esports |
| `8` | Olympics & multi-sport |

### Crypto (`primary_category = 3`)

**Horizon** (short-term price / oracle-style buckets):

| `subcategory` | Topic |
|----------------|--------|
| `1` | 5-minute horizon |
| `2` | 15-minute horizon |
| `3` | 1-hour horizon |
| `4` | 4-hour horizon |
| `5` | Daily horizon |
| `6` | Weekly horizon |
| `7` | End-of-year / long dated |
| `8` | Other / custom horizon |

**Theme** (use **distinct id ranges** so horizon vs theme does not collide — pick one scheme per market):

| `subcategory` | Topic |
|----------------|--------|
| `20` | Spot / index price |
| `21` | Regulation & ETFs |
| `22` | Protocol / chain (TVL, upgrades) |
| `23` | Security incidents (hacks, exploits) |

If a market needs **both** horizon and theme, store the primary drill-down in `subcategory` and push the other axis to **off-chain tags**, or reserve a future on-chain `tags` bitmap (not part of this layout).

### Weather (`primary_category = 4`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | Temperature (records, thresholds) |
| `2` | Precipitation (rain / snow totals) |
| `3` | Wind & storms (hurricanes, cyclones) |
| `4` | Air quality |
| `5` | Volcanic activity |
| `6` | Earthquakes & tsunamis |
| `7` | Seasonal / climate (ENSO, drought) |

### Economics (`primary_category = 5`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | Central banks & rates |
| `2` | Inflation & prices |
| `3` | Employment |
| `4` | Growth & recession |

### Companies & tech (`primary_category = 6`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | Earnings |
| `2` | Product launches |
| `3` | M&A |

### Science & health (`primary_category = 7`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | Clinical / FDA |
| `2` | Space |
| `3` | Energy & climate science |

### Culture & entertainment (`primary_category = 8`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | Film & TV |
| `2` | Music |
| `3` | Awards |

### World & disasters (`primary_category = 9`)

| `subcategory` | Topic |
|----------------|--------|
| `1` | Conflict |
| `2` | Humanitarian |

---

## Instruction Arguments

Categories are written at creation time:

| Instruction | Sets |
|-------------|------|
| [CreateMarket](../program/instructions.md#0---createmarket) | `Market.primary_category`, `Market.subcategory` |
| [CreateEvent](../program/instructions.md#9---createevent) | `Event.primary_category`, `Event.subcategory` |

Optional future instructions (`UpdateMarketCategory`, etc.) are **not** specified here; changing categories post-create affects indexer caches and UI assumptions.

---

## Filtering Cheatsheet

Offsets assume **Borsh** layout from [Account Structs](../program/accounts.md). Combine with the account discriminant filter (`Market` = `0`, `Event` = `3`).

| Goal | Memcmp offset | Bytes |
|------|----------------|-------|
| All markets in primary **Crypto** (`3`) | `291` | single byte `3` |
| All events in primary **Weather** (`4`) | `588` | single byte `4` |

Subcategory filters require **two-byte little-endian** `subcategory` at `292` (markets) or `589` (events). Many RPC clients encode memcmp as base64 over raw bytes — serialize `subcategory` as `u16::to_le_bytes()`.

---

## Upgrades & Governance

- Adding new **primary** ids (`10`+) or **subcategory** rows does **not** require resizing accounts.
- Renaming or merging categories is **off-chain** documentation only; ids stay stable.
- If the program later **rejects** unknown ids, bump the instruction version and document migration for uncategorized (`0`) markets.

---

## Next Steps

- [Account Structs — Market](../program/accounts.md#market-account)
- [Account Structs — Event](../program/accounts.md#event-account)
- [Markets](./markets.md)
- [Events](./events.md)
