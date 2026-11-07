/**
 * Panggilan ke backend Rust wallet (`apps/wallet/src-tauri/src/commands`) lewat IPC Tauri. Jumlah koin selalu teks
 * angka quanta. Galat dari Rust berbentuk `{ code, message }`.
 */

import { invoke } from '@tauri-apps/api/core';
import type { WalletNetwork } from './address';
import type { HistoryEntryDto } from './history';

export interface WalletInfo {
  exists: boolean;
  path: string;
  defaultRpcUrl: string;
}

export interface ChainStatus {
  network: string;
  height: number;
  bestBlockId: string;
}

export interface BalanceDto {
  tipHeight: number;
  total: string;
  spendable: string;
  immature: string;
  pendingSpent: string;
  pendingReceived: string;
  afterPending: string;
  utxos: number;
}

export interface HistoryDto {
  tipHeight: number;
  entries: HistoryEntryDto[];
}

export interface SentDto {
  txid: string;
  amount: string;
  fee: string;
  change: string | null;
  inputs: number;
  size: number;
}

export interface WalletError {
  code: string;
  message: string;
}

export function toWalletError(error: unknown): WalletError {
  if (typeof error === 'object' && error !== null && 'code' in error && 'message' in error) {
    const { code, message } = error as { code: unknown; message: unknown };
    if (typeof code === 'string' && typeof message === 'string') {
      return { code, message };
    }
  }
  return { code: 'failed', message: String(error) };
}

export const ipc = {
  walletInfo: (network: WalletNetwork) => invoke<WalletInfo>('wallet_info', { network }),
  newPhrase: () => invoke<string>('new_phrase'),
  restoreWallet: (network: WalletNetwork, phrase: string, password: string, confirm: string) =>
    invoke<string>('restore_wallet', { network, phrase, password, confirm }),
  receiveAddress: (network: WalletNetwork) => invoke<string>('receive_address', { network }),
  newAddress: (network: WalletNetwork) => invoke<string>('new_address', { network }),
  chainStatus: (network: WalletNetwork, rpcUrl: string) => invoke<ChainStatus>('chain_status', { network, rpcUrl }),
  balance: (network: WalletNetwork, rpcUrl: string) => invoke<BalanceDto>('balance', { network, rpcUrl }),
  history: (network: WalletNetwork, rpcUrl: string) => invoke<HistoryDto>('history', { network, rpcUrl }),
  send: (args: {
    network: WalletNetwork;
    rpcUrl: string;
    password: string;
    to: string;
    amount: string;
    feeRate: number;
  }) => invoke<SentDto>('send', args),
  revealPhrase: (network: WalletNetwork, password: string) =>
    invoke<string>('reveal_phrase', { network, password }),
};
