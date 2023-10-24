# EVM

> [!NOTE]
> You can find the contract's ABI in [this compilation artifact](artifacts/contracts/AccessGrants.sol/AccessGrants.json#L5).

## Testing

```
$ yarn test
```

## Local deployment

First, start a [local Hardhat node](https://hardhat.org/hardhat-runner/docs/getting-started#connecting-a-wallet-or-dapp-to-hardhat-network) on a terminal.

```
$ npx hardhat node

Started HTTP and WebSocket JSON-RPC server at http://127.0.0.1:8545/
...
```

On another terminal, run the deployment script.

```
$ npx hardhat run --network localhost scripts/deploy.js

AccessGrants deployed to 0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0
```

You can then use the [Hardhat console](https://hardhat.org/hardhat-runner/docs/guides/hardhat-console) to interact with the locally deployed contract. Make sure to use the address you deployed to.

```
$ npx hardhat console --network localhost

> const accessGrants = await ethers.getContractAt("AccessGrants", "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0");

> await accessGrants.findGrants("0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0", "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0", "no");
Result(0) []
...
```
