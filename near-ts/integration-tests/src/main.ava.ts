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

  const grants_for:Array<Object> = await contract.view("grants_for", {
    grantee: "bob.near",
    dataId: "90",
  });

  t.deepEqual(grants_for, [
    { owner: "test.near", grantee: "bob.near", dataId: "90", lockedUntil: "0" },
  ]);

  let grants_by:Array<Object>;

  grants_by = await contract.view("grants_by", {
    grantee: "charlie.near",
    dataId: "90",
  });

  t.deepEqual(grants_by, []);

  grants_by = await contract.view("grants_by", {
    owner: "test.near",
  });

  t.deepEqual(grants_by, [
    { owner: "test.near", grantee: "bob.near", dataId: "42", lockedUntil: "0" },
    { owner: "test.near", grantee: "bob.near", dataId: "90", lockedUntil: "0" },
    { owner: "test.near", grantee: "charlie.near", dataId: "99", lockedUntil: "0" },
  ]);

  await root.call(contract, "delete_grant", {
    grantee: "bob.near",
    dataId: "90",
  });

  grants_by = await contract.view("grants_by", {
    owner: "test.near",
  });

  t.deepEqual(grants_by, [
    { owner: "test.near", grantee: "bob.near", dataId: "42", lockedUntil: "0" },
    { owner: "test.near", grantee: "charlie.near", dataId: "99", lockedUntil: "0" },
  ]);

  await root.call(contract, "insert_grant", {
    grantee: "dave.near",
    dataId: "99",
    lockedUntil: (Date.now() + 24*60*60*1000) * 1e6
  });

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: "dave.near",
    dataId: "99",
  });

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant is timelocked"));
});
