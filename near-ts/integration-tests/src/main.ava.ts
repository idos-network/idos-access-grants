import { Worker, NearAccount, TransactionResult } from "near-workspaces";
import anyTest, { TestFn } from "ava";
import { PublicKey, PublicKeyString } from "../../contract/src/public_key";

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
  publicKeys: Record<string, PublicKeyString>;
}>;

test.beforeEach(async (t) => {
  // Init the worker and start a Sandbox server
  const worker = await Worker.init();

  // Deploy contract
  const root = worker.rootAccount;
  const contract = await root.createSubAccount("test-account");
  // Get wasm file path from package.json test script in folder above
  await contract.deploy(
    process.argv[2],
  );

  // Save state for test runs, it is unique for each test
  t.context.worker = worker;
  t.context.accounts = { root, contract };

  for(const name of ["bob", "charlie", "dave", "eve"]){
    t.context.accounts[name] = await worker.rootAccount.createSubAccount(name)
  }

  t.context.publicKeys = {}
  for(const [name, account] of Object.entries(t.context.accounts)){
    t.context.publicKeys[name] = ((await account.getKey())!.getPublicKey()! as unknown as PublicKey).toString()
  }

});

test.afterEach.always(async (t) => {
  // Stop Sandbox server
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to stop the Sandbox:", error);
  });
});

test("everything", async (t) => {
  const { root: testAccount, contract } = t.context.accounts;
  const { bob, charlie, dave, root: test, eve } = t.context.publicKeys;
  let transactionResult: TransactionResult;

  await testAccount.call(contract, "insert_grant", {
    grantee: bob,
    dataId: "42",
  });

  transactionResult = await testAccount.callRaw(contract, "insert_grant", {
    grantee: bob,
    dataId: "42",
  });

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant already exists"));

  await testAccount.call(contract, "insert_grant", {
    grantee: bob,
    dataId: "90",
  });

  await testAccount.call(contract, "insert_grant", {
    grantee: charlie,
    dataId: "99",
  });

  await testAccount.call(contract, "insert_grant", {
    grantee: dave,
    dataId: "99",
  });

  const grants_for:Array<Object> = await contract.view("grants_for", {
    grantee: bob,
    dataId: "90",
  });

  t.deepEqual(grants_for, [
    { owner: test, grantee: bob, dataId: "90", lockedUntil: "0" },
  ]);

  let foundGrants:Array<Object>;

  foundGrants = await contract.view("find_grants", {
    grantee: charlie,
    dataId: "90",
  });

  t.deepEqual(foundGrants, []);

  foundGrants = await contract.view("find_grants", {
    grantee: charlie,
    dataId: "99",
  });

  t.deepEqual(foundGrants, [
    { owner: test, grantee: charlie, dataId: "99", lockedUntil: "0" },
  ]);

  foundGrants = await contract.view("find_grants", {
    owner: test,
  });

  t.deepEqual(foundGrants, [
    { owner: test, grantee: bob, dataId: "42", lockedUntil: "0" },
    { owner: test, grantee: bob, dataId: "90", lockedUntil: "0" },
    { owner: test, grantee: charlie, dataId: "99", lockedUntil: "0" },
    { owner: test, grantee: dave, dataId: "99", lockedUntil: "0" },
  ]);


  await testAccount.call(contract, "delete_grant", {
    grantee: charlie,
    dataId: "99",
  });

  foundGrants = await contract.view("find_grants", {
    owner: test,
  });

  t.deepEqual(foundGrants, [
    { owner: test, grantee: bob, dataId: "42", lockedUntil: "0" },
    { owner: test, grantee: bob, dataId: "90", lockedUntil: "0" },
    { owner: test, grantee: dave, dataId: "99", lockedUntil: "0" },
  ]);

  foundGrants = await contract.view("find_grants", {
    owner: test,
    dataId: "99",
  });

  t.deepEqual(foundGrants, [
    { owner: test, grantee: dave, dataId: "99", lockedUntil: "0" },
  ]);

  await t.throwsAsync(() => (
    contract.view("find_grants", {
      dataId: "99",
    })
  ), {message: /Required argument: `owner` and\/or `grantee`/});


  /*
   * Timelock expired
   */

  let lockedUntil = (Date.now() - 24*60*60*1000) * 1e6;

  await testAccount.call(contract, "insert_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await testAccount.callRaw(contract, "delete_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: eve,
  });

  t.assert(foundGrants.length == 0);

  await testAccount.call(contract, "insert_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await testAccount.callRaw(contract, "delete_grant", {
    grantee: eve,
    dataId: "99",
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: eve,
  });

  t.assert(foundGrants.length == 0);

  transactionResult = await testAccount.callRaw(contract, "delete_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: 0,
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: eve,
  });

  t.assert(foundGrants.length == 0);


  /*
   * Still timelocked
   */

  lockedUntil = (Date.now() + 24*60*60*1000) * 1e6;

  await testAccount.call(contract, "insert_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await testAccount.callRaw(contract, "delete_grant", {
    grantee: eve,
    dataId: "99",
    lockeduntil: lockedUntil,
  });

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant is timelocked"));

  foundGrants = await contract.view("find_grants", {
    grantee: eve,
  });

  t.assert(foundGrants.length == 1);

  transactionResult = await testAccount.callRaw(contract, "delete_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: 0,
  });

  foundGrants = await contract.view("find_grants", {
    grantee: eve,
  });

  t.assert(foundGrants.length == 1);

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant is timelocked"));
});
