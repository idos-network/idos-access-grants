require('dotenv-flow').config();
require("@nomicfoundation/hardhat-toolbox");

module.exports = {
  solidity: "0.8.19",
  networks: {
    sepolia: {
      url: process.env.SEPOLIA_NODE_URL,
      accounts: [process.env.SEPOLIA_PRIVATE_KEY]
    }
  }
};
