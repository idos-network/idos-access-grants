// Find all our documentation at https://docs.near.org
import {
  NearBindgen,
  near,
  call,
  view,
  LookupMap,
  encode,
  decode,
  assert,
} from "near-sdk-js";
import { PublicKey, PublicKeyString } from "./public_key";

class Grant {
  owner: PublicKeyString;
  grantee: PublicKeyString;
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
    grantee: PublicKeyString,
    dataId: string,
    lockedUntil: bigint
  }): void {
    const owner = new PublicKey(near.signerAccountPk()).toString();
    lockedUntil = lockedUntil || 0n;

    const grant = new Grant({ owner, grantee, dataId, lockedUntil });

    const grantId = this.deriveGrantId({ grant });

    assert(this.grantsById.get(grantId) === null, "Grant already exists");

    this.grantsById.set(grantId, grant);

    this.grantIdsByOwner.set(
      owner,
      (this.grantIdsByOwner.get(owner) || []).concat(grantId),
    );
    this.grantIdsByGrantee.set(
      grantee,
      (this.grantIdsByGrantee.get(grantee) || []).concat(grantId),
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
    grantee: PublicKeyString,
    dataId: string,
    lockedUntil: bigint
  }): void {
    const owner = new PublicKey(near.signerAccountPk()).toString();
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
        this.grantIdsByGrantee.set(
          grantee,
          (this.grantIdsByGrantee.get(grantee) || []).filter((id) => (id !== grantId)),
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
    grantee: PublicKeyString,
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
    owner: PublicKeyString,
    grantee: PublicKeyString,
    dataId: string,
  }): Grant[] {
    assert(owner || grantee, "Required argument: `owner` and/or `grantee`");

    const grantIdSearches = [
      this.grantIdsByOwner.get(owner),
      this.grantIdsByGrantee.get(grantee),
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

    const grantId = decode(
      near.keccak256(
        encode(owner + grantee + dataId + lockedUntil),
      ),
    );

    return grantId;
  }
}
