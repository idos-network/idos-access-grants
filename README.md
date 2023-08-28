# idos-access-grants

# tl;dr:

- **a grant** is an IDOS object representing a data access grant from an owner to a grantee for a given data ID (optionally with a timelock)
- the contract **stores a collection of grants**
- **anyone** can **list grants**
- a **signer** can
    - **create a grant** that they own
    - **delete a grant** that they own (unless timelocked)

# Interface

## Objects

<details><summary><h3><code>Grant</code></h3></summary>

Represents an access grant from a data owner, to a grantee, for a given data ID, until a given time.

**Variables**
- `owner`: address
- `grantee`: address
- `dataId`: string
- `lockedUntil`: 256-bit unsigned integer

</details>

## Functions

<details><summary><h3><code>insertGrant</code></h3></summary>

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

<details><summary><h3><code>deleteGrant</code></h3></summary>

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

<details><summary><h3><code>findGrants</code></h3></summary>

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

<details><summary><h3><code>grantsFor</code></h3></summary>

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






