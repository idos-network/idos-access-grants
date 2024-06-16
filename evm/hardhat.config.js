require("dotenv-flow").config();
require('@openzeppelin/hardhat-upgrades');
require("@nomicfoundation/hardhat-toolbox");
require("@nomicfoundation/hardhat-verify");

module.exports = {
  solidity: {
    version: "0.8.20",
    settings: {
      optimizer: {
        enabled: true,
        runs: 1000,
      },
    },
  },
  networks: {
    hardhat: {},
    sepolia: {
      url: process.env.SEPOLIA_NODE_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
    arbitrumSepolia: {
      url: process.env.ARBITRUM_SEPOLIA_NODE_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
    arbitrumOne: {
      url: process.env.ARBITRUM_ONE_NODE_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
    etherlink: {
      url: "https://node.mainnet.etherlink.com",
      accounts: [process.env.PRIVATE_KEY],
    },
    etherlinkTestnet: {
      url: "https://node.ghostnet.etherlink.com",
      accounts: [process.env.PRIVATE_KEY],
    }
  },
  etherscan: {
    apiKey: {
      sepolia: process.env.SEPOLIA_ETHERSCAN_API_KEY,
      arbitrumSepolia: process.env.ARBITRUM_ETHERSCAN_API_KEY,
      arbitrumOne: process.env.ARBITRUM_ETHERSCAN_API_KEY,
      etherlink: "YOU_CAN_COPY_ME",
      etherlinkTestnet: "YOU_CAN_COPY_ME",
    },
    customChains: [
      {
        network: "etherlink",
        chainId: 42793,
        urls: {
          apiURL: "https://explorer.etherlink.com/api",
          browserURL: "https://explorer.etherlink.com"
        }
      },
      {
        network: "etherlinkTestnet",
        chainId: 128123,
        urls: {
          apiURL: "https://testnet-explorer.etherlink.com/api",
          browserURL: "https://testnet-explorer.etherlink.com"
        }
      },
    ]
  }
};
