const { ethers, upgrades } = require("hardhat");

async function main() {
    if(!process.env.PROXY_ADDRESS) throw new Error("Missing PROXY_ADDRESS env var");

    const AccessGrantsV2 = await ethers.getContractFactory("AccessGrantsV2");
    const accessGrantsV2 = await upgrades.upgradeProxy(process.env.PROXY_ADDRESS, AccessGrantsV2, { opts: {} });
    console.log(`${process.env.PROXY_ADDRESS} upgraded to AccessGrantsV2`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
