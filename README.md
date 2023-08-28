# idos-access-grants

When receiving a signed request for data not owned by the signer, idOS nodes use these smart contracts as the source of truth for authorizing (or denying) the request.

The contract functionality is straightforward:

- **a grant** is an IDOS object representing a data access grant from an owner to a grantee for a given data ID (optionally with a timelock)
- the contract **stores a collection of grants**
- **anyone** can **list grants**
- a **signer** can
    - **create a grant** that they own
    - **delete a grant** that they own (unless timelocked)

## Contracts

**Implementations:**

| Target VM | Language | Source |
| :- | :- | :- |
| EVM | Solidity | [evm](evm) |
| NEAR VM | Rust | [near-rs](near-rs) |
| NEAR VM | TypeScript | [near-ts](near-ts) |

**Deployments:**

| Chain | Address |
| :- | :- |
| Goerli | [`0x56272dd0C469B2D831a86a81f3E1eeCb0bDe8850`](https://goerli.etherscan.io/address/0x56272dd0c469b2d831a86a81f3e1eecb0bde8850#code) |
| NEAR Testnet | [`dev-20230822145635-43013572991135`](https://explorer.testnet.near.org/accounts/dev-20230822145635-43013572991135) |

## Interface

**ℹ️ Note:** while this abstract interface description uses mixedCase, the implementations above follow the style guide of the respective language. For example, in EVM + Solidity we use mixedCase (`insertGrant`), while in NEAR VM + Rust/TypeScript we use snake_case (`insert_grant`).

### Objects

<details><summary><h4><code>Grant</code></h4></summary>

Represents an access grant from a data owner, to a grantee, for a given data ID, until a given time.

**Variables**
- `owner`: address
- `grantee`: address
- `dataId`: string
- `lockedUntil`: 256-bit unsigned integer

</details>

### Functions

<details><summary><h4><code>insertGrant</code></h4></summary>

Creates a new access grant.

**Arguments**

- required
  - `grantee`: address
  - `dataId`: string
- optional
  - `lockedUntil`: 256-bit unsigned integer

**Implements**

- creates `Grant(signer, grantee, dataId, lockedUntil)`
- reverts if this grant already exists

</details>

<details><summary><h4><code>deleteGrant</code></h4></summary>

Deletes an existing access grant.

**Arguments**

- required
  - `grantee`: address
  - `dataId`: string
- optional
  - `lockedUntil`: 256-bit unsigned integer

**Implements**

- if given `lockedUntil`
    - deletes `Grant(signer, grantee, dataId, lockedUntil)`
    - reverts if `lockedUntil` is in the past
- else
    - deletes all `Grant(signer, grantee, dataId, *)`
    - reverts if any `lockedUntil` is in the past

</details>

<details><summary><h4><code>findGrants</code></h4></summary>

Lists grants matching the provided arguments.

**Arguments**

- required (both or either)
  - `owner`: address
  - `grantee`: address
- optional
  - `dataId`: string

**Implements**

Performs a wildcard search, matching existing grants to given arguments, which must follow one of these patterns:

```
{ owner, grantee, dataId }
{ owner, grantee, ****** }
{ owner, *******, dataId }
{ owner, *******, ****** }
{ *****, grantee, dataId }
{ *****, grantee, ****** }
```

**Returns**

A list of 0+ `Grant`s

</details>

<details><summary><h4><code>grantsFor</code></h4></summary>

Lists grants matching the provided arguments.

**Arguments**

- required
  - `grantee`: address
  - `dataId`: string

**Implements**

Calls `grantsBy` with no `owner` argument.

**Returns**

A list of 0+ `Grant`s

</details>
