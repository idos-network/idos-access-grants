require("dotenv-flow").config();
require("@nomicfoundation/hardhat-toolbox");
require("@nomicfoundation/hardhat-verify");

module.exports = {
  solidity: "0.8.19",
  networks: {
    sepolia: {
      url: process.env.SEPOLIA_NODE_URL,
      accounts: [process.env.SEPOLIA_PRIVATE_KEY],
    },
  },
  etherscan: {
    apiKey: {
      sepolia: process.env.SEPOLIA_ETHERSCAN_API_KEY,
    },
  },
  sourcify: {
    enabled: true,
  },
};
