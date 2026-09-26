import { describe, expect, it } from "vitest";
import { configData, parsePlayer, tag } from "./program";
describe("ABI encoder",()=>{it("encodes little-endian tags",()=>expect(tag(3).toString("hex")).toBe("0300000000000000"));it("requires 10000 basis points",()=>expect(()=>configData([{name:"A",weight:9999}])).toThrow());it("parses player state",()=>{const b=Buffer.alloc(32);b[2]=2;b[4]=3;b.writeBigInt64LE(7n,8);expect(parsePlayer(b)).toMatchObject({status:2,prize:3,day:7})});it("rejects malformed state",()=>expect(()=>parsePlayer(Buffer.alloc(2))).toThrow())});
