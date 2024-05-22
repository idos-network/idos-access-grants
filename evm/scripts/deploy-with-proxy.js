const { ethers, upgrades } = require("hardhat");

async function main() {
  const AccessGrantsV1 = await ethers.getContractFactory("AccessGrantsV1");
  const accessGrantsV1 = await upgrades.deployProxy(AccessGrantsV1, [], { kind: "uups" });
  await accessGrantsV1.waitForDeployment();

  console.log("AccessGrants deployed, proxy is at:", await accessGrantsV1.getAddress());
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
