import { PublicKey, SystemProgram, TransactionInstruction } from "@solana/web3.js";
export const PROGRAM_ID=new PublicKey(process.env.NEXT_PUBLIC_PROGRAM_ID||"E86BYgK5ZUVNwegBJWXxyCk79XB6dsB5debp3QuuEa9Q");
export const VRF_PROGRAM=new PublicKey("Vrf1RNUjXmQGjmQrQLvJHs9SNkvDJEsRVFPkfSQUwGz");
export const VRF_QUEUE=new PublicKey("Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh");
export const tag=(n:number)=>{const b=Buffer.alloc(8);b.writeBigUInt64LE(BigInt(n));return b};
export const configPda=()=>PublicKey.findProgramAddressSync([Buffer.from("config")],PROGRAM_ID)[0];
export const playerPda=(wallet:PublicKey)=>PublicKey.findProgramAddressSync([Buffer.from("player"),wallet.toBuffer()],PROGRAM_ID)[0];
export const identityPda=()=>PublicKey.findProgramAddressSync([Buffer.from("identity")],PROGRAM_ID)[0];
export function initPlayerIx(wallet:PublicKey){return new TransactionInstruction({programId:PROGRAM_ID,keys:[{pubkey:wallet,isSigner:true,isWritable:true},{pubkey:playerPda(wallet),isSigner:false,isWritable:true},{pubkey:SystemProgram.programId,isSigner:false,isWritable:false}],data:tag(2)})}
export function drawIx(wallet:PublicKey,game:0|1){return new TransactionInstruction({programId:PROGRAM_ID,keys:[{pubkey:wallet,isSigner:true,isWritable:true},{pubkey:playerPda(wallet),isSigner:false,isWritable:true},{pubkey:configPda(),isSigner:false,isWritable:false},{pubkey:VRF_QUEUE,isSigner:false,isWritable:true},{pubkey:identityPda(),isSigner:false,isWritable:true},{pubkey:VRF_PROGRAM,isSigner:false,isWritable:false},{pubkey:new PublicKey("SysvarS1otHashes111111111111111111111111111"),isSigner:false,isWritable:false},{pubkey:SystemProgram.programId,isSigner:false,isWritable:false}],data:Buffer.concat([tag(3),Buffer.from([game,crypto.getRandomValues(new Uint8Array(1))[0]])])})}
export type Prize={name:string;weight:number};
export function configData(prizes:Prize[]){if(!prizes.length||prizes.length>8||prizes.reduce((s,p)=>s+p.weight,0)!==10000)throw new Error("概率总和必须为 100%（10000 基点）");const parts=[tag(1),Buffer.from([prizes.length])];for(const p of prizes){const w=Buffer.alloc(2);w.writeUInt16LE(p.weight);const n=Buffer.alloc(16);n.write(p.name.slice(0,16));parts.push(w,n)}return Buffer.concat(parts)}
export function parsePlayer(d:Buffer){if(d.length!==32)throw new Error("玩家账户长度错误");return{status:d[2],game:d[3],prize:d[4],day:Number(d.readBigInt64LE(8)),nonce:Number(d.readBigUInt64LE(16))}}
