"use client";
import { useMemo } from "react";
import { ConnectionProvider, WalletProvider } from "@solana/wallet-adapter-react";
import { WalletModalProvider } from "@solana/wallet-adapter-react-ui";
import "@solana/wallet-adapter-react-ui/styles.css";
export function Wallets({children}:{children:React.ReactNode}) { const endpoint=useMemo(()=>process.env.NEXT_PUBLIC_RPC_URL||"https://api.devnet.solana.com",[]); return <ConnectionProvider endpoint={endpoint}><WalletProvider wallets={[]} autoConnect><WalletModalProvider>{children}</WalletModalProvider></WalletProvider></ConnectionProvider> }
