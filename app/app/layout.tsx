import type { Metadata } from "next";
import "./globals.css";
import { Wallets } from "./wallets";
export const metadata: Metadata = { title: "幸运花园", description: "适合小朋友的公平抽奖小游戏" };
export default function Layout({children}:{children:React.ReactNode}) { return <html lang="zh-CN"><body><Wallets>{children}</Wallets></body></html> }
