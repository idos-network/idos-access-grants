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
    }
  },
  etherscan: {
    apiKey: {
      sepolia: process.env.SEPOLIA_ETHERSCAN_API_KEY,
      arbitrumSepolia: process.env.ARBITRUM_ETHERSCAN_API_KEY,
      arbitrumOne: process.env.ARBITRUM_ETHERSCAN_API_KEY,
    },
  },
  sourcify: {
    enabled: false,
  },
};
