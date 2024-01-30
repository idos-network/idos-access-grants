// Find all our documentation at https://docs.near.org
import {
  NearBindgen,
  near,
  call,
  view,
  LookupMap,
  AccountId,
  encode,
  decode,
  assert,
  PublicKey,
  CurveType,
  UnknownCurve,
} from "near-sdk-js";
import { base58, hex } from "@scure/base";

export function curveTypeToStr(value: CurveType): string {
  switch (value) {
      case CurveType.ED25519:
          return "ed25519";
      case CurveType.SECP256K1:
          return "secp256k1";
      default:
          throw new UnknownCurve();
  }
}

// pkoch: I got to these two vmPublicKey functions by running JSON.stringify on what I got inside the NEAR VM. Since
// that's what I assume is running on the blockchain, even though it doesn't match the code in the SDK, I'm going
// to assume the VM is the relevant truth.
// biome-ignore lint/suspicious/noExplicitAny: see above
const vmPublicKeyCurveType = (pk: PublicKey): CurveType => (pk as any).keyType as CurveType
// biome-ignore lint/suspicious/noExplicitAny: see above
const vmPublicKeyData = (pk: PublicKey): Uint8Array => new Uint8Array((pk.data as any).data)

const publicKeyToString = (pk: PublicKey): string => {
  const curveStr = curveTypeToStr(vmPublicKeyCurveType(pk))
  const encoded = base58.encode(vmPublicKeyData(pk))
  return `${curveStr}:${encoded}`;
}

const publicKeyToUint8Array = (pk: PublicKey): Uint8Array => {
  return new Uint8Array((function* () {
    yield vmPublicKeyCurveType(pk);

    const data = vmPublicKeyData(pk);
    for(let i = 0; i < data.length; i++) {
      yield data.at(i);
    }
  })())
}

class Grant {
  owner: AccountId;
  grantee: PublicKey;
  dataId: string;
  lockedUntil: bigint;

  constructor({ owner, grantee, dataId, lockedUntil }: Grant) {
    this.owner = owner;
    this.grantee = grantee;
    this.dataId = dataId;
    this.lockedUntil = lockedUntil;
  }
}

@NearBindgen({})
export class AccessGrants {
  grantsById: LookupMap<Grant>;

  grantIdsByOwner: LookupMap<string[]>;
  grantIdsByGrantee: LookupMap<string[]>;
  grantIdsByDataId: LookupMap<string[]>;

  constructor() {
    this.grantsById = new LookupMap<Grant>("grants_by_id_");

    this.grantIdsByOwner = new LookupMap<string[]>("grant_ids_by_owner_");
    this.grantIdsByGrantee = new LookupMap<string[]>("grant_ids_by_grantee_");
    this.grantIdsByDataId = new LookupMap<string[]>("grant_ids_by_data_id_");
  }

  @call({})
  insert_grant({
    grantee,
    dataId,
    lockedUntil
  }: {
    grantee: PublicKey,
    dataId: string,
    lockedUntil: bigint
  }): void {
    const owner = near.signerAccountId();
    lockedUntil = lockedUntil || 0n;

    const grant = new Grant({ owner, grantee, dataId, lockedUntil });

    const grantId = this.deriveGrantId({ grant });

    assert(this.grantsById.get(grantId) === null, "Grant already exists");

    this.grantsById.set(grantId, grant);

    this.grantIdsByOwner.set(
      owner,
      (this.grantIdsByOwner.get(owner) || []).concat(grantId),
    );
    const granteeString = publicKeyToString(grantee);
    this.grantIdsByGrantee.set(
      granteeString,
      (this.grantIdsByGrantee.get(granteeString) || []).concat(grantId),
    );
    this.grantIdsByDataId.set(
      dataId,
      (this.grantIdsByDataId.get(dataId) || []).concat(grantId),
    );
  }

  @call({})
  delete_grant({
    grantee,
    dataId,
    lockedUntil
  }: {
    grantee: PublicKey,
    dataId: string,
    lockedUntil: bigint
  }): void {
    const owner = near.signerAccountId();
    lockedUntil = lockedUntil || 0n;

    const grants = this.find_grants({ owner, grantee, dataId });

    grants.forEach((grant) => {
      if (lockedUntil == 0n || grant.lockedUntil == lockedUntil) {
        assert(near.blockTimestamp() > grant.lockedUntil, "Grant is timelocked");

        const grantId = this.deriveGrantId({ grant });

        this.grantsById.remove(grantId);

        this.grantIdsByOwner.set(
          owner,
          (this.grantIdsByOwner.get(owner) || []).filter((id) => (id !== grantId)),
        );
        const granteeString = publicKeyToString(grantee);
        this.grantIdsByGrantee.set(
          granteeString,
          (this.grantIdsByGrantee.get(granteeString) || []).filter((id) => (id !== grantId)),
        );
        this.grantIdsByDataId.set(
          dataId,
          (this.grantIdsByDataId.get(dataId) || []).filter((id) => (id !== grantId)),
        );
      }
    });
  }

  @view({})
  grants_for({
    grantee,
    dataId,
  }: {
    grantee: PublicKey,
    dataId: string,
  }): Grant[] {
    return this.find_grants({ owner: null, grantee, dataId });
  }


  @view({})
  find_grants({
    owner,
    grantee,
    dataId,
  }: {
    owner: AccountId,
    grantee: PublicKey,
    dataId: string,
  }): Grant[] {
    assert(owner || grantee, "Required argument: `owner` and/or `grantee`");

    const granteeString = publicKeyToString(grantee);
    const grantIdSearches = [
      this.grantIdsByOwner.get(owner),
      this.grantIdsByGrantee.get(granteeString),
      this.grantIdsByDataId.get(dataId),
    ];

    const grants = grantIdSearches
      .filter(Array.isArray)
      .reduce((acc, val) => (acc.filter((id) => (val.includes(id)))))
      .map((id) => (this.grantsById.get(id)));

    return grants;
  }

  @view({})
  deriveGrantId({
    grant
  }: {
    grant: Grant
  }): string {
    const { owner, grantee, dataId, lockedUntil } = grant;

    const grantId = hex.encode(
      near.keccak256(
        encode(owner + publicKeyToString(grantee) + dataId + lockedUntil),
      ),
    );

    return grantId;
  }
}
