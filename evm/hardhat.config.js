require("dotenv-flow").config();
require("@nomicfoundation/hardhat-toolbox");
require("@nomicfoundation/hardhat-verify");

module.exports = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: {
        enabled: true,
        runs: 1000,
      },
    },
  },
  networks: {
    sepolia: {
      url: process.env.SEPOLIA_NODE_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
    arbitrumSepolia: {
      url: process.env.ARBITRUM_SEPOLIA_NODE_URL,
      accounts: [process.env.PRIVATE_KEY],
    },
  },
  etherscan: {
    apiKey: {
      sepolia: process.env.SEPOLIA_ETHERSCAN_API_KEY,
      arbitrumSepolia: process.env.ARBITRUM_SEPOLIA_ETHERSCAN_API_KEY,
    },
  },
  sourcify: {
    enabled: true,
  },
};
