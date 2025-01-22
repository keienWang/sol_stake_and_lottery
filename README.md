# 🚀 Solana Staking & Lottery Contract

This project implements a **staking and lottery system on Solana** using the **Anchor framework**. It consists of two main smart contracts:

- **Staking Contract (`gdtc_staking`)**: Users can stake LP tokens and earn rewards.
- **Lottery Contract (`gdtc_lottery`)**: A lottery system where users participate using LP tokens, and winners receive token rewards.

---

## 📌 Features

### 🎯 Staking System
- ✅ Users can **stake LP tokens** and earn **GDTC rewards**.
- ✅ Supports **fixed staking plans**.
- ✅ Implements **referral rewards** via `user_superior_token_account`.

### 🎲 Lottery System
- 🎟️ Users join the lottery **by staking 1 LP token**.
- 🎯 The lottery is **automatically triggered** when **50 LPs** are staked.
- 🏆 Winner is determined by a **hashed value comparison**.
- 🎁 **10 GDTC tokens** are rewarded to the winner.
- ⛏️ The **staked 50 LPs** are sent to a **3-month mining pool** for yield farming.
- 💰 After 3 months, users can **withdraw their LP**, and remaining mining rewards go to the **foundation**.

--- 