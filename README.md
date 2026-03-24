# Afristore

> Decentralized marketplace for African art, built on Stellar + Soroban smart contracts.

## Architecture

```
Freighter Wallet ──► Next.js Frontend ──► Soroban Contract (Stellar)
                          │                       │
                          ▼                       ▼
                     Pinata IPFS             On-chain Storage
                  (Images + Metadata)     (Listings + Ownership)
```

## Monorepo Structure

```
afristore/
├── contracts/
│   └── soroban-marketplace/          # Rust Soroban smart contract
│       ├── src/
│       │   ├── lib.rs                # Contract entry point
│       │   ├── types.rs              # Listing, Status, Error types
│       │   ├── storage.rs            # Storage key helpers
│       │   └── contract.rs           # Contract implementation
│       ├── Cargo.toml
│       └── Makefile
├── frontend/
│   └── afristore-app/                # Next.js 14 App Router frontend
│       ├── src/
│       │   ├── app/                  # App Router pages
│       │   ├── components/           # Reusable UI components
│       │   ├── lib/                  # Stellar SDK, IPFS, contract helpers
│       │   └── hooks/                # React hooks
│       ├── public/
│       ├── package.json
│       └── next.config.js
└── scripts/
    └── deploy/                       # Deployment scripts
        ├── deploy_contract.sh
        └── fund_account.sh
```

## Quick Start

### 1. Deploy the Soroban contract (Testnet)

```bash
cd scripts/deploy
./fund_account.sh          # fund a new keypair on testnet
./deploy_contract.sh       # build + deploy the contract
```

### 2. Start the frontend

```bash
cd frontend/afristore-app
cp .env.example .env.local   # fill in contract ID + Pinata keys
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

## Environment Variables

| Variable | Description |
|---|---|
| `NEXT_PUBLIC_CONTRACT_ID` | Deployed Soroban contract address |
| `NEXT_PUBLIC_STELLAR_NETWORK` | `testnet` or `mainnet` |
| `NEXT_PUBLIC_STELLAR_RPC_URL` | Soroban RPC endpoint |
| `NEXT_PUBLIC_STELLAR_HORIZON_URL` | Horizon API endpoint |
| `NEXT_PUBLIC_PINATA_JWT` | Pinata JWT for IPFS uploads |
| `NEXT_PUBLIC_PINATA_GATEWAY` | Pinata IPFS gateway URL |

## Tech Stack

| Layer | Technology |
|---|---|
| Frontend | Next.js 14 (App Router) |
| Blockchain | Stellar / Soroban |
| Smart Contracts | Rust (soroban-sdk) |
| Wallet | Freighter |
| Storage | IPFS via Pinata |
| Blockchain SDK | @stellar/stellar-sdk |

## Future Improvements

- PostgreSQL event indexer for fast queries + search
- On-chain royalties (EIP-2981 equivalent for Soroban)
- Secondary resale market with royalty split
- Search + category filtering via indexed metadata
- Auction / bidding contract
- Analytics dashboard
- Mobile wallet support (LOBSTR, xBull)


## Future Plan (V2)

### 1) Split into dedicated repositories

To scale development and deployment, this monorepo will be split into focused repos:

- `afristore-frontend`
    - Next.js app, wallet UX, listing/discovery pages, creator dashboard
- `afristore-backend`
    - Indexer, API, search, analytics, notifications, admin services
- `afristore-contracts`
    - Soroban marketplace, auction, royalty, and protocol-level smart contracts

### 2) Marketplace evolution

- Move from “create NFT in marketplace” to “list existing NFT for sale”
- Enable clean primary and secondary sales flow
- Preserve `original_creator` + royalty rules across all resales
- Keep protocol fee and payout splitting fully on-chain

### 3) Launchpad creation

The launchpad will support creator-first primary drops:

- Collection/project setup for artists and brands
- Configurable drop mechanics (fixed price, timed drop, allowlist)
- Primary mint + instant listing pipeline
- Launch metrics dashboard (mints, volume, conversion)
- Shared royalty and treasury configuration with marketplace

### 4) Supporting improvements

- PostgreSQL event indexer for fast queries and search
- Auction / bidding contracts and offer workflows
