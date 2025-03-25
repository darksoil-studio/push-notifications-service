import { assert, test } from "vitest";

import { ActionHash, Delete, Record, SignedActionHashed } from "@holochain/client";
import { dhtSync, runScenario } from "@holochain/tryorama";
import { decode } from "@msgpack/msgpack";
import { toPromise } from "@tnesh-stack/signals";
import { EntryRecord } from "@tnesh-stack/utils";
import { cleanNodeDecoding } from "@tnesh-stack/utils/dist/clean-node-decoding.js";

import { sampleServiceAccountKey } from "../../ui/src/mocks.js";
import { ServiceAccountKey } from "../../ui/src/types.js";
import { setup } from "./setup.js";

test("create ServiceAccountKey", async () => {
  await runScenario(async scenario => {
    const [alice, bob] = await setup(scenario);

    // Alice creates a ServiceAccountKey
    const serviceAccountKey: EntryRecord<ServiceAccountKey> = await alice.store.client.createServiceAccountKey(
      await sampleServiceAccountKey(alice.store.client),
    );
    assert.ok(serviceAccountKey);
  });
});

test("create and read ServiceAccountKey", async () => {
  await runScenario(async scenario => {
    const [alice, bob] = await setup(scenario);

    const sample = await sampleServiceAccountKey(alice.store.client);

    // Alice creates a ServiceAccountKey
    const serviceAccountKey: EntryRecord<ServiceAccountKey> = await alice.store.client.createServiceAccountKey(sample);
    assert.ok(serviceAccountKey);

    // Wait for the created entry to be propagated to the other node.
    await dhtSync(
      [alice.player, bob.player],
      alice.player.cells[0].cell_id[0],
    );

    // Bob gets the created ServiceAccountKey
    const createReadOutput: EntryRecord<ServiceAccountKey> = await toPromise(
      bob.store.serviceAccountKeys.get(serviceAccountKey.actionHash).entry,
    );
    assert.deepEqual(sample, cleanNodeDecoding(createReadOutput.entry));
  });
});

test("create and delete ServiceAccountKey", async () => {
  await runScenario(async scenario => {
    const [alice, bob] = await setup(scenario);

    // Alice creates a ServiceAccountKey
    const serviceAccountKey: EntryRecord<ServiceAccountKey> = await alice.store.client.createServiceAccountKey(
      await sampleServiceAccountKey(alice.store.client),
    );
    assert.ok(serviceAccountKey);

    // Alice deletes the ServiceAccountKey
    const deleteActionHash = await alice.store.client.deleteServiceAccountKey(serviceAccountKey.actionHash);
    assert.ok(deleteActionHash);

    // Wait for the created entry to be propagated to the other node.
    await dhtSync(
      [alice.player, bob.player],
      alice.player.cells[0].cell_id[0],
    );

    // Bob tries to get the deleted ServiceAccountKey
    const deletes: Array<SignedActionHashed<Delete>> = await toPromise(
      bob.store.serviceAccountKeys.get(serviceAccountKey.actionHash).deletes,
    );
    assert.equal(deletes.length, 1);
  });
});
