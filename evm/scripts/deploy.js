const hre = require("hardhat");

async function main() {
  const accessGrants = await hre.ethers.deployContract("AccessGrants");

  await accessGrants.waitForDeployment();

  console.log(`AccessGrants deployed to ${accessGrants.target}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
