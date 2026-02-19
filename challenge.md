# NFT Staking Core — Challenge

Build on top of the existing Core NFT staking program or create your own from scratch.
You are free to modify existing methods, state accounts, or logic.

---

## Task 1: Core Plugins (Mandatory)

### 1. Claim Rewards Without Unstaking
Create a `claim_rewards` instruction that lets users collect accumulated rewards without unstaking their NFT.

**Requirements:**
- Mint reward tokens to the user's ATA
- Keep the NFT staked and frozen


### 2. Burn-to-Earn with BurnDelegate

Create a `burn_staked_nft` instruction that lets users permanently burn their staked NFT for a massive one-time reward bonus.

**Requirements:**
- Mint reward tokens to the user's ATA
- Burn the NFT


### 3. Collection-Level Staking Stats (Attributes on Collection)

Track staking statistics at the collection level using Attributes on the Collection account itself.

**Requirements:**
- Add a `"total_staked"` counter as an Attribute on the Collection account
- Increment on stake, decrement on unstake

---

## Task 2: Oracle Plugin (Optional)

Implement an external plugin.

### Time-Based Trading

NFTs can only be traded during specific hours (e.g., 9AM-5PM UTC). Outside these hours, trading is blocked.

**Requirements:**
- Create an Oracle Account to save the Approved/Rejected per lifecycle event
- Add the Oracle Plugin adapter to your Collection (on creation or later)
- Write a cron that updates your Oracle account to toggle transfer Approved/Rejected

