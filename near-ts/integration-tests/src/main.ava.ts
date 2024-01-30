import { Worker, NearAccount, TransactionResult, PublicKey } from "near-workspaces";
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

test("derive_grant_id_example is the same as in Rust", async (t) => {
  const { root, contract } = t.context.accounts;

  t.deepEqual(
    await contract.view("deriveGrantId", {
      grant: {
        owner: "my-cool-account.near",
        grantee: PublicKey.fromString("ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp"),
        dataId: "some data",
        lockedUntil: 1337,
      }
    }),
    "8f3254135e2aeb003e10443cb54e2ca4830862fc4e774f6d98715d9668210caa",
  )
});

test("everything", async (t) => {
  const { root, contract } = t.context.accounts;
  let transactionResult: TransactionResult;

  const bob = PublicKey.from("ed25519:337mMe7y3aobP3GJGcDYJBxuZoLbcs1SzLYaAt5dNjnG");
  const charlie = PublicKey.from("ed25519:GCrv4FBs5QyKwGm1sEX5suNeDFuWUkW4Gh9DqHnXr3ro");
  const dave = PublicKey.from("ed25519:9Vaeikpf6tF7smXvkEcXimST5Vw7u7idZTUDjFLBx3i4");
  const eve = PublicKey.from("ed25519:8vjCFQe5C4crR66gYoBb1NkmuBMZPjLNNXg27MuCCmbm");

  await root.call(contract, "insert_grant", {
    grantee: bob,
    dataId: "42",
  });

  transactionResult = await root.callRaw(contract, "insert_grant", {
    grantee: bob,
    dataId: "42",
  });

  t.assert(transactionResult.failed);
  t.assert(transactionResult.receiptFailureMessagesContain("Grant already exists"));

  await root.call(contract, "insert_grant", {
    grantee: bob,
    dataId: "90",
  });

  await root.call(contract, "insert_grant", {
    grantee: charlie,
    dataId: "99",
  });

  await root.call(contract, "insert_grant", {
    grantee: dave,
    dataId: "99",
  });

  const grants_for:Array<Object> = await contract.view("grants_for", {
    grantee: bob,
    dataId: "90",
  });

  t.deepEqual(grants_for, [
    { owner: "test.near", grantee: bob, dataId: "90", lockedUntil: "0" },
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
    { owner: "test.near", grantee: charlie, dataId: "99", lockedUntil: "0" },
  ]);

  foundGrants = await contract.view("find_grants", {
    owner: "test.near",
  });

  t.deepEqual(foundGrants, [
    { owner: "test.near", grantee: bob, dataId: "42", lockedUntil: "0" },
    { owner: "test.near", grantee: bob, dataId: "90", lockedUntil: "0" },
    { owner: "test.near", grantee: charlie, dataId: "99", lockedUntil: "0" },
    { owner: "test.near", grantee: dave, dataId: "99", lockedUntil: "0" },
  ]);


  await root.call(contract, "delete_grant", {
    grantee: charlie,
    dataId: "99",
  });

  foundGrants = await contract.view("find_grants", {
    owner: "test.near",
  });

  t.deepEqual(foundGrants, [
    { owner: "test.near", grantee: bob, dataId: "42", lockedUntil: "0" },
    { owner: "test.near", grantee: bob, dataId: "90", lockedUntil: "0" },
    { owner: "test.near", grantee: dave, dataId: "99", lockedUntil: "0" },
  ]);

  foundGrants = await contract.view("find_grants", {
    owner: "test.near",
    dataId: "99",
  });

  t.deepEqual(foundGrants, [
    { owner: "test.near", grantee: dave, dataId: "99", lockedUntil: "0" },
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
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: eve,
  });

  t.assert(foundGrants.length == 0);

  await root.call(contract, "insert_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await root.callRaw(contract, "delete_grant", {
    grantee: eve,
    dataId: "99",
  });

  t.assert(transactionResult.succeeded);

  foundGrants = await contract.view("find_grants", {
    grantee: eve,
  });

  t.assert(foundGrants.length == 0);

  transactionResult = await root.callRaw(contract, "delete_grant", {
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

  await root.call(contract, "insert_grant", {
    grantee: eve,
    dataId: "99",
    lockedUntil: lockedUntil,
  });

  transactionResult = await root.callRaw(contract, "delete_grant", {
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

  transactionResult = await root.callRaw(contract, "delete_grant", {
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
