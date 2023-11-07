import {
  PublicKey as OriginalPublicKey,
  CurveType,
  UnknownCurve,
  concat
} from "near-sdk-js";

import { base58 } from "@scure/base";

export class PublicKey extends OriginalPublicKey {
  toString(): PublicKeyString {
    return `${curveTypeFromStr(this.curveType())}:${base58.encode(this.data.slice(1))}` as PublicKeyString;
  }
}

export function curveTypeFromStr(value: CurveType): string {
  switch (value) {
      case CurveType.ED25519:
          return "ed25519";
      case CurveType.SECP256K1:
          return "secp256k1";
      default:
          throw new UnknownCurve();
  }
}

export type PublicKeyString = string & {readonly PublicKeyString: unique symbol}
