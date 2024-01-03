require("@nomicfoundation/hardhat-toolbox");

const SEPOLIA_PRIVATE_KEY = "b8d88a575069fd56f60618bceb68ca30e8643e948ed9cf8e82230b31edc47269";

module.exports = {
  solidity: "0.8.19",
  networks: {
    sepolia: {
      url: "https://sepolia.infura.io/v3/87def88bcc0a4760af5c6279e1e2599b",
      accounts: [SEPOLIA_PRIVATE_KEY]
    }
  }
};
