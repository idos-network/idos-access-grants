const { loadFixture } = require("@nomicfoundation/hardhat-network-helpers");
const { expect } = require("chai");

const WILDCARD_ADDRESS = ethers.ZeroAddress;
const WILDCARD_DATA = "0";

describe("AccessGrants", function () {
  async function deployAndPopulateContractFixture() {
    const [, signer1, signer2, signer3, signer4] = await ethers.getSigners();

    const AccessGrants = await ethers.getContractFactory("AccessGrants");
    const accessGrants = await AccessGrants.deploy();

    await accessGrants.connect(signer1).insert_grant(signer2, "1A");
    await accessGrants.connect(signer1).insert_grant(signer2, "1B");
    await accessGrants.connect(signer1).insert_grant(signer3, "1A");

    await accessGrants.connect(signer2).insert_grant(signer1, "2A");
    await accessGrants.connect(signer2).insert_grant(signer3, "2A");


    return { accessGrants, signer1, signer2, signer3, signer4 };
  }

  describe("Grant management", function () {
    describe("Creating grants", function () {
      it("A grant can only be owned by its creator", async function () {
        const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

        await accessGrants.connect(owner).insert_grant(grantee, "some ID");

        let grants = await accessGrants.grants_by(owner, grantee, "some ID");

        expect(grants.length).to.equal(1);
        expect(grants[0].owner).to.equal(owner.address);
      });

      it("A duplicate grant cannot be created", async function () {
        const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

        await expect(
          accessGrants.connect(owner).insert_grant(grantee, "1A")
        ).to.be.revertedWith("Grant already exists");
      });
    });

    describe("Deleting grants", function () {
      it("A grant can be deleted by its owner", async function () {
        const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

        await accessGrants.connect(owner).insert_grant(grantee, "some ID");
        await accessGrants.connect(owner).delete_grant(grantee, "some ID");

        let grants = await accessGrants.grants_by(owner, grantee, "some ID");

        expect(grants.length).to.equal(0);
      });

      it("A grant cannot be deleted by anyone else", async function () {
        const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

        await accessGrants.connect(owner).insert_grant(grantee, "some ID");

        await expect(
          accessGrants.connect(grantee).delete_grant(grantee, "some ID")
        ).to.be.revertedWith("No grants for sender");

        let grants = await accessGrants.grants_by(owner, grantee, "some ID");

        expect(grants.length).to.equal(1);
      });
    });

    describe("Reading grants", function () {
      describe("grants_for", async function () {
        describe("When the grant doesn't exist", async function () {
          it("Returns no grants", async function () {
            const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

            await accessGrants.connect(owner).insert_grant(grantee, "some ID");

            let grants = await accessGrants.grants_for(grantee, "bad ID");

            expect(grants.length).to.equal(0);
            expect(grants).to.eql([]);
          });
        });

        describe("When the grant exists", async function () {
          it("Returns the grant, regardless of caller", async function () {
            const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

            await accessGrants.connect(owner).insert_grant(grantee, "some ID");

            let grants1 = await accessGrants.connect(owner).grants_for(grantee, "some ID");
            let grants2 = await accessGrants.connect(grantee).grants_for(grantee, "some ID");

            expect(grants1.length).to.equal(1);
            expect(grants1).to.eql([
              [owner.address, grantee.address, "some ID", 0n],
            ]);
            expect(grants1).to.eql(grants2);
          });
        });
      });

      describe("grants_by", async function () {
        describe("When the grant doesn't exist", async function () {
          it("Returns no grants", async function () {
            const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

            await accessGrants.connect(owner).insert_grant(grantee, "some ID");

            let grants = await accessGrants.grants_by(owner, grantee, "bad ID");

            expect(grants.length).to.equal(0);
            expect(grants).to.eql([]);
          });
        });

        describe("When grants exists", async function () {
          it("Returns the grant, regardless of caller", async function () {
            const { accessGrants, signer1: owner, signer2: grantee } = await loadFixture(deployAndPopulateContractFixture);

            await accessGrants.connect(owner).insert_grant(grantee, "some ID");

            let grants1 = await accessGrants.connect(owner).grants_by(owner, grantee, "some ID");
            let grants2 = await accessGrants.connect(grantee).grants_by(owner, grantee, "some ID");

            expect(grants1.length).to.equal(1);
            expect(grants1).to.eql([
              [owner.address, grantee.address, "some ID", 0n],
            ]);
            expect(grants1).to.eql(grants2);
          });

          describe("With wildcard arguments", async function () {
            it("Reverts if neither owner nor grantee provided", async function () {
              const { accessGrants } = await loadFixture(deployAndPopulateContractFixture);

              await expect(
                accessGrants.grants_by(WILDCARD_ADDRESS, WILDCARD_ADDRESS, "some ID")
              ).to.be.revertedWith("Neither owner nor grantee provided");
            });

            describe("Given owner", async function () {
              it("Returns grants for any grantee, any data", async function () {
                const { accessGrants, signer1, signer2, signer3 } = await loadFixture(deployAndPopulateContractFixture);

                let grants = await accessGrants.grants_by(signer1, WILDCARD_ADDRESS, WILDCARD_DATA);

                expect(grants.length).to.equal(3);
                expect(grants).to.eql([
                  [signer1.address, signer2.address, "1A", 0n],
                  [signer1.address, signer2.address, "1B", 0n],
                  [signer1.address, signer3.address, "1A", 0n],
                ]);

                grants = await accessGrants.grants_by(signer2, WILDCARD_ADDRESS, WILDCARD_DATA);

                expect(grants.length).to.equal(2);
                expect(grants).to.eql([
                  [signer2.address, signer1.address, "2A", 0n],
                  [signer2.address, signer3.address, "2A", 0n],
                ]);
              });

              it("Returns grants for given grantee, any data", async function () {
                const { accessGrants, signer1, signer2, signer3 } = await loadFixture(deployAndPopulateContractFixture);

                let grants = await accessGrants.grants_by(signer1, signer2, WILDCARD_DATA);

                expect(grants.length).to.equal(2);
                expect(grants).to.eql([
                  [signer1.address, signer2.address, "1A", 0n],
                  [signer1.address, signer2.address, "1B", 0n],
                ]);

                grants = await accessGrants.grants_by(signer1, signer3, WILDCARD_DATA);

                expect(grants.length).to.equal(1);
                expect(grants).to.eql([
                  [signer1.address, signer3.address, "1A", 0n],
                ]);
              });

              it("Returns grants for any grantee, given data", async function () {
                const { accessGrants, signer1, signer2, signer3 } = await loadFixture(deployAndPopulateContractFixture);

                let grants = await accessGrants.grants_by(signer1, WILDCARD_ADDRESS, "1A");

                expect(grants.length).to.equal(2);
                expect(grants).to.eql([
                  [signer1.address, signer2.address, "1A", 0n],
                  [signer1.address, signer3.address, "1A", 0n],
                ]);

                grants = await accessGrants.grants_by(signer2, WILDCARD_ADDRESS, "2A");

                expect(grants.length).to.equal(2);
                expect(grants).to.eql([
                  [signer2.address, signer1.address, "2A", 0n],
                  [signer2.address, signer3.address, "2A", 0n],
                ]);
              });
            });


            describe("Given grantee", async function () {
              it("Returns grants for any ownwer, any data", async function () {
                const { accessGrants, signer1, signer2, signer3 } = await loadFixture(deployAndPopulateContractFixture);

                let grants = await accessGrants.grants_by(WILDCARD_ADDRESS, signer1, WILDCARD_DATA);

                expect(grants.length).to.equal(1);
                expect(grants).to.eql([
                  [signer2.address, signer1.address, "2A", 0n],
                ]);

                grants = await accessGrants.grants_by(WILDCARD_ADDRESS, signer3, WILDCARD_DATA);

                expect(grants.length).to.equal(2);
                expect(grants).to.eql([
                  [signer1.address, signer3.address, "1A", 0n],
                  [signer2.address, signer3.address, "2A", 0n],
                ]);
              });

              it("Returns grants for any ownwer, given data", async function () {
                const { accessGrants, signer1, signer2, signer3 } = await loadFixture(deployAndPopulateContractFixture);

                let grants = await accessGrants.grants_by(WILDCARD_ADDRESS, signer2, "1A");

                expect(grants.length).to.equal(1);
                expect(grants).to.eql([
                  [signer1.address, signer2.address, "1A", 0n],
                ]);

                grants = await accessGrants.grants_by(WILDCARD_ADDRESS, signer3, "2A");

                expect(grants.length).to.equal(1);
                expect(grants).to.eql([
                  [signer2.address, signer3.address, "2A", 0n],
                ]);
              });
            });
          });
        });
      });
    });
  });
});
