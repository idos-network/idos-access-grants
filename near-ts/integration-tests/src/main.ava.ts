import { Worker, NearAccount, TransactionResult } from "near-workspaces";
import anyTest, { TestFn } from "ava";

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
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
});

test.afterEach.always(async (t) => {
  // Stop Sandbox server
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed to stop the Sandbox:", error);
  });
});

test("everything", async (t) => {
  const { root, contract } = t.context.accounts;
  let transactionResult: TransactionResult;

  await root.call(contract, "insert_grant", {
    grantee: "bob.near",
    dataId: "42",
  });

  transactionResult = await root.callRaw(contract, "insert_grant", {
    grantee: "bob.near",
    dataId: "42",
  });

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant already exists"));

  await root.call(contract, "insert_grant", {
    grantee: "bob.near",
    dataId: "90",
  });

  await root.call(contract, "insert_grant", {
    grantee: "charlie.near",
    dataId: "99",
  });

  await root.call(contract, "insert_grant", {
    grantee: "dave.near",
    dataId: "99",
  });

  const grants_for:Array<Object> = await contract.view("grants_for", {
    grantee: "bob.near",
    dataId: "90",
  });

  t.deepEqual(grants_for, [
    { owner: "test.near", grantee: "bob.near", dataId: "90", lockedUntil: "0" },
  ]);

  let foundGrants:Array<Object>;

  foundGrants = await contract.view("find_grants", {
    grantee: "charlie.near",
    dataId: "90",
  });

  t.deepEqual(foundGrants, []);

  foundGrants = await contract.view("find_grants", {
    grantee: "charlie.near",
    dataId: "99",
  });

  t.deepEqual(foundGrants, [
    { owner: "test.near", grantee: "charlie.near", dataId: "99", lockedUntil: "0" },
  ]);

  foundGrants = await contract.view("find_grants", {
    owner: "test.near",
  });

  t.deepEqual(foundGrants, [
    { owner: "test.near", grantee: "bob.near", dataId: "42", lockedUntil: "0" },
    { owner: "test.near", grantee: "bob.near", dataId: "90", lockedUntil: "0" },
    { owner: "test.near", grantee: "charlie.near", dataId: "99", lockedUntil: "0" },
    { owner: "test.near", grantee: "dave.near", dataId: "99", lockedUntil: "0" },
  ]);


  await root.call(contract, "delete_grant", {
    grantee: "charlie.near",
    dataId: "99",
  });

  foundGrants = await contract.view("find_grants", {
    owner: "test.near",
  });

  t.deepEqual(foundGrants, [
    { owner: "test.near", grantee: "bob.near", dataId: "42", lockedUntil: "0" },
    { owner: "test.near", grantee: "bob.near", dataId: "90", lockedUntil: "0" },
    { owner: "test.near", grantee: "dave.near", dataId: "99", lockedUntil: "0" },
  ]);

  foundGrants = await contract.view("find_grants", {
    owner: "test.near",
    dataId: "99",
  });

  t.deepEqual(foundGrants, [
    { owner: "test.near", grantee: "dave.near", dataId: "99", lockedUntil: "0" },
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

  await root.call(contract, "insert_grant", {
    grantee: "eve.near",
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: "eve.near",
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: "eve.near",
  });

  t.assert(foundGrants.length == 0);

  await root.call(contract, "insert_grant", {
    grantee: "eve.near",
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: "eve.near",
    dataId: "99",
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: "eve.near",
  });

  t.assert(foundGrants.length == 0);

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: "eve.near",
    dataId: "99",
    lockedUntil: 0,
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: "eve.near",
  });

  t.assert(foundGrants.length == 0);


  /*
   * Still timelocked
   */

  lockedUntil = (Date.now() + 24*60*60*1000) * 1e6;

  await root.call(contract, "insert_grant", {
    grantee: "eve.near",
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: "eve.near",
    dataId: "99",
    lockeduntil: lockedUntil,
  });

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant is timelocked"));

  foundGrants = await contract.view("find_grants", {
    grantee: "eve.near",
  });

  t.assert(foundGrants.length == 1);

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: "eve.near",
    dataId: "99",
    lockedUntil: 0,
  });

  foundGrants = await contract.view("find_grants", {
    grantee: "eve.near",
  });

  t.assert(foundGrants.length == 1);

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant is timelocked"));
});
