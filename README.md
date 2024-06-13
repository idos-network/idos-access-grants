# idOS access grant contracts

![EVM](https://img.shields.io/badge/EVM-gray?logo=ethereum) ![NEAR](https://img.shields.io/badge/NEAR%20VM-gray?logo=near) ![License](https://img.shields.io/badge/license-MIT-blue?&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI2NHB4IiBoZWlnaHQ9IjY0cHgiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSIjRkZGRkZGIiBzdHJva2Utd2lkdGg9IjIiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCI+PGcgaWQ9IlNWR1JlcG9fYmdDYXJyaWVyIiBzdHJva2Utd2lkdGg9IjAiPjwvZz48ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiPjwvZz48ZyBpZD0iU1ZHUmVwb19pY29uQ2FycmllciI+IDxwYXRoIGQ9Ik0xNiAxNmwzLTggMy4wMDEgOEE1LjAwMiA1LjAwMiAwIDAxMTYgMTZ6Ij48L3BhdGg+IDxwYXRoIGQ9Ik0yIDE2bDMtOCAzLjAwMSA4QTUuMDAyIDUuMDAyIDAgMDEyIDE2eiI+PC9wYXRoPiA8cGF0aCBkPSJNNyAyMWgxMCI+PC9wYXRoPiA8cGF0aCBkPSJNMTIgM3YxOCI+PC9wYXRoPiA8cGF0aCBkPSJNMyA3aDJjMiAwIDUtMSA3LTIgMiAxIDUgMiA3IDJoMiI+PC9wYXRoPiA8L2c+PC9zdmc+Cg==)

When receiving a signed request for data not owned by the signer, idOS nodes use these smart contracts as the source of truth for authorizing (or denying) the request.

The contract functionality is straightforward:

- **a grant** is an idOS object representing a data access grant from an owner to a grantee for a given data ID (optionally with a timelock)
- the contract **stores a collection of grants**
- **anyone** can **list grants**
- a **signer** can
    - **create a grant** that they own
    - **delete a grant** that they own (unless timelocked)

## Contracts

**Implementations:**

| Target VM | Language | Source |
| :- | :- | :- |
| EVM | Solidity | [`evm/`](./evm) |
| NEAR VM | Rust | [`near-rs/`](./near-rs) |

**Deployments:**

| Source | Chain | Address |
| :- | :- | :- |
| `evm/` | Sepolia | [`0xA5Ac9B9703Bd661cd2aC05B41FE57d1A5DD332AA`](https://sepolia.etherscan.io/address/0xA5Ac9B9703Bd661cd2aC05B41FE57d1A5DD332AA#code) |
| `evm/` | Arbitrum Sepolia | [`0x350829c8FCb3DF16EeaE9ADDa2565090348426f9`](https://sepolia.arbiscan.io/address/0x350829c8FCb3DF16EeaE9ADDa2565090348426f9#code) |
| `evm/` | Arbitrum One | [`0x350829c8FCb3DF16EeaE9ADDa2565090348426f9`](https://arbiscan.io/address/0x350829c8FCb3DF16EeaE9ADDa2565090348426f9#code) |
| `near-rs/` | NEAR Testnet | [`idos-dev-4.testnet`](https://explorer.testnet.near.org/accounts/idos-dev-4.testnet) |
| `near-rs/` | NEAR Mainnet | [`idos-dev-4.near`](https://explorer.mainnet.near.org/accounts/idos-dev-4.near) |

### Deploy to Sepolia

1. Copy `.env` file to `.env.local` and fill it in accordingly
1. Run `npx hardhat --network sepolia run scripts/deploy.js`
1. Run `npx hardhat --network sepolia verify $RESULTING_ADDRESS`

### Deploy to local chain

Use [hardhat](https://hardhat.org/) to run local node.
1. Run node in separate process `npx hardhat node`
1. Compile a contract `npx hardhat compile`
1. Deploy the contract `npx hardhat --network locahost run scripts/deploy.js`


## Interface

> [!NOTE]
> This interface description uses mixedCase, but each implementation follows the respective language's style guide, e.g.:
> * in EVM + Solidity, we use mixedCase (`insertGrant`)
> * in NEAR VM + Rust/TypeScript, we use snake_case (`insert_grant`).

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
    - reverts if `lockedUntil` is in the future
- else
    - deletes all `Grant(signer, grantee, dataId, *)`
    - reverts if any `lockedUntil` is in the future

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
