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

Implement one or both options of external plugin examples.

### Option A: Whitelist-Based Staking

Only specific NFT owners can stake their NFTs. An admin maintains a whitelist of approved addresses.

**Requirements:**
- Create an Oracle account that stores a list of whitelisted addresses
- Add the Oracle Plugin adapter to your Collection
- Admin can add/remove addresses from the whitelist
- When staking, check if the user is whitelisted


#### Option B: Time-Based Staking

NFTs can only be staked during specific hours (e.g., 9AM-5PM UTC). Outside these hours, staking is blocked.

**Requirements:**
- Create an Oracle account that stores current staking status (allowed/blocked)
- Add the Oracle Plugin adapter to your Collection
- Write a cron that writes and updates to your Oracle Plugin to toggle staking on/off

**Resources:**
- [Oracle Plugin Documentation](https://developers.metaplex.com/smart-contracts/core/external-plugins/oracle)


