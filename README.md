Prediction CLOB
===============

A high-performance decentralized prediction market built on the Solana blockchain utilizing a Central Limit Order Book (CLOB) architecture.

Description
-----------

This project implements a binary outcome prediction market where users can trade outcome tokens using a peer-to-peer order book. It leverages Solana's high-throughput capabilities and Anchor's zero\_copy account structures to manage large-scale financial state with minimal latency and maximum capital efficiency.

Architecture
------------
<img width="821" height="527" alt="Screenshot 2026-05-10 222447" src="https://github.com/user-attachments/assets/e9dd4b22-a564-4347-8369-e011c99b8a57" />
</br>
<img width="887" height="906" alt="Screenshot 2026-05-10 221937" src="https://github.com/user-attachments/assets/90673b5e-0563-40dc-a662-3b00091e40e6" />


*   **Market Genesis:** Creator initializes the Market PDA for metadata and a PDA-owned Token Vault for collateral escrow.
    
*   **Orderbook Allocation:** Large 90KB+ accounts are pre-allocated client-side to hold 1024 nodes in bidirectional linked lists.
    
*   **User Onboarding:** UserAccount PDA tracks "Escrowed Collateral" (funds in open orders) using seeds \[b"user", market, wallet\].
    
*   **Lazy Tokenization:** Outcome ATAs are created via idempotent frontend instructions only when a user intends to hold shares.
    
*   **Matching Engine:** Price-time priority logic crossings trigger mint\_to of outcome tokens directly to user wallets.
    
*   **Collateral Escrow:** Funds move from User ATA to the Market Vault upon order placement; makers receive quote deltas upon fill.
    
*   **Platform Fees:** A u64 counter tracks unclaimed fees in the Orderbook, allowing for efficient bulk sweeps to a global fee account.
    
*   **Settlement Freeze:** Finalization sets the winning outcome and transitions the program to a "Settled" state, disabling all trade instructions.
    
*   **Liquidation/Claiming:** Users burn winning tokens through a claim\_collateral instruction to receive their portion of the Market Vault.
    
*   **Account Cleanup:** Rent-exempt lamports are reclaimed by closing UserAccount and Market PDAs once balances are zeroed.

Project Structure
-----------------


    prediction-clob/
    ├── programs/prediction-clob/src/
    │   ├── instructions/
    │   │   ├── initialize_market.rs      # Creates the central market state
    │   │   ├── initialize_orderbook.rs   # Setup for 90KB+ zero-copy structures
    │   │   ├── initialize_user_account.rs # Creates the portfolio/ledger PDA
    │   │   ├── place_order.rs            # Handles trading and matching calls
    │   │   ├── claim_funds.rs            # Collateral withdrawal logic
    │   │   ├── resolution.rs             # Market settlement and outcome finalization
    │   │   └── mod.rs
    │   ├── logic/
    │   │   ├── matching.rs               # Core matching engine and execution
    │   │   ├── linked_list.rs            # Node traversal and sorted insertion
    │   │   └── mod.rs
    │   ├── state.rs                      # Account structs (Market, Orderbook, UserAccount)
    │   ├── quantities.rs                 # Domain-specific wrappers (Ticks, BaseLots)
    │   └── lib.rs                        # Program entry points
    ├── tests/
    │   └── prediction_test.ts            # End-to-end integration tests for all instructions
    ├── migrations/                       # Deployment scripts
    └── Anchor.toml                       # Workspace configuration


### Account Model

*   **Market Account:** The central state for a specific prediction event. It stores metadata, authority, settlement deadlines, and public keys for the collateral and outcome mints.
    
*   **User Account:** A Program Derived Address (PDA) unique to each user-market pair. It tracks the user's current positions (Outcome A/B balances) and claimable collateral.
    
*   **Orderbook Account:** A large (90KB+) zero-copy account representing the bid and ask sides for a specific outcome. It is pre-allocated on the client side to bypass the 10KB CPI limit.
    

### Order Management

The Orderbook maintains a fixed-size array of 1024 OrderNode entries. These nodes are organized into three distinct linked lists:

1.  **Bid List:** Sorted by price (descending) to ensure best execution.
    
2.  **Ask List:** Sorted by price (ascending).
    
3.  **Free List:** Unused nodes available for new orders, tracked via a free\_head pointer to enable O(1) allocation.
    

### Matching Engine

The engine utilizes a price-time priority algorithm. When an order is placed, it immediately attempts to cross the spread against the opposing linked list. Remaining quantities are inserted into the appropriate sorted list.

Key Implementations
-------------------

*   **Zero-Copy Serialization:** Uses AccountLoader and #\[repr(C)\] to map on-chain data directly to Rust memory layouts, avoiding the high CPU cost of Borsh serialization for large arrays.
    
*   **Linked-List Logic:** Implements O(n) worst-case insertion and O(1) removal. Sorting ensures that the matching engine only needs to check the list head for potential matches.
    
*   **PDA Strategy:** Deterministic seeds \[b"user", market, wallet\] and \[b"market", market\_id\] allow for efficient state lookup without requiring client-side indexing.
    
*   **Memory Alignment:** Strict use of padding and u64 alignment in structs to ensure compatibility with the Solana Virtual Machine (SVM).
    

Assumptions
-----------

*   **Fixed Orderbook Size:** The system is currently capped at 1024 orders per side to maintain predictable compute unit consumption.
    
*   **Tick Precision:** Prices are handled as integers (Ticks), assuming a fixed decimal precision defined at the market level.
    
*   **Collateral Asset:** The market operates using a single SPL Token as collateral (e.g., USDC).
    
*   **Settlement:** An external authority or oracle is responsible for proposing and finalizing the winning outcome.
