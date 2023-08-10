const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");
const { expect } = require("chai");

const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";
const ZERO_DATA_ID = "0";

describe("AccessGrants", function () {
  async function deployContractFixture() {
    const [, signer1, signer2, signer3, signer4] = await ethers.getSigners();
    
    const AccessGrants = await ethers.getContractFactory("AccessGrants");
    const accessGrants = await AccessGrants.deploy();

    return { accessGrants, signer1, signer2, signer3, signer4 };
  }

  describe("Grant management", function () {
    it("Works", async function () {
      const { accessGrants, signer1, signer2, signer3, signer4 } = await loadFixture(deployContractFixture);

      await accessGrants.connect(signer1).insert_grant(signer2, "1A");
      await accessGrants.connect(signer1).insert_grant(signer2, "1B");
      await accessGrants.connect(signer1).insert_grant(signer3, "1A");

      await accessGrants.connect(signer2).insert_grant(signer1, "2A");
      await accessGrants.connect(signer2).insert_grant(signer3, "2A");

      await accessGrants.connect(signer2).insert_grant(signer3, "2B");
      await accessGrants.connect(signer2).delete_grant(signer3, "2B");

      /*
       * grants_for
       */

      let grants = await accessGrants.grants_for(signer2, "1A");
      expect(grants.length).to.equal(1);
      expect(grants).to.eql([
        [signer1.address, signer2.address, "1A", 0n],
      ]);

      grants = await accessGrants.grants_for(signer3, "1B");
      expect(grants.length).to.equal(0);

      grants = await accessGrants.grants_for(signer3, "2B");
      expect(grants.length).to.equal(0);

      grants = await accessGrants.grants_for(signer4, "1B");
      expect(grants.length).to.equal(0);

      /*
       * grants_by
       */

      grants = await accessGrants.grants_by(signer1, ZERO_ADDRESS, ZERO_DATA_ID);

      expect(grants.length).to.equal(3);
      expect(grants).to.eql([
        [signer1.address, signer2.address, "1A", 0n],
        [signer1.address, signer2.address, "1B", 0n],
        [signer1.address, signer3.address, "1A", 0n],
      ]);

      grants = await accessGrants.grants_by(signer2, ZERO_ADDRESS, ZERO_DATA_ID);

      expect(grants.length).to.equal(2);
      expect(grants).to.eql([
        [signer2.address, signer1.address, "2A", 0n],
        [signer2.address, signer3.address, "2A", 0n],
      ]);

      grants = await accessGrants.grants_by(signer1, signer2, ZERO_DATA_ID);
      expect(grants.length).to.equal(2);
      expect(grants).to.eql([
        [signer1.address, signer2.address, "1A", 0n],
        [signer1.address, signer2.address, "1B", 0n],
      ]);

      grants = await accessGrants.grants_by(signer1, signer2, "1A");
      expect(grants.length).to.equal(1);
      expect(grants).to.eql([
        [signer1.address, signer2.address, "1A", 0n],
      ]);

      grants = await accessGrants.grants_by(signer1, ZERO_ADDRESS, "1A");
      expect(grants.length).to.equal(2);
      expect(grants).to.eql([
        [signer1.address, signer2.address, "1A", 0n],
        [signer1.address, signer3.address, "1A", 0n],
      ]);

      grants = await accessGrants.grants_by(signer1, ZERO_ADDRESS, "1C");
      expect(grants.length).to.equal(0);

      grants = await accessGrants.grants_by(signer3, ZERO_ADDRESS, "2B");
      expect(grants.length).to.equal(0);
    });
  });
});
