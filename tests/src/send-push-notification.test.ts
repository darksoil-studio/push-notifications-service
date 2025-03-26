import {
	ActionHash,
	Delete,
	Record,
	SignedActionHashed,
} from '@holochain/client';
import { dhtSync, runScenario } from '@holochain/tryorama';
import { decode } from '@msgpack/msgpack';
import { toPromise } from '@tnesh-stack/signals';
import { EntryRecord } from '@tnesh-stack/utils';
import { cleanNodeDecoding } from '@tnesh-stack/utils/dist/clean-node-decoding.js';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { assert, test } from 'vitest';

// import { setup } from './setup.js';

const serviceProviderHapp =
	dirname(fileURLToPath(import.meta.url)) +
	'/../../workdir/push-notifications-service-provider.happ';

const endUserHapp =
	dirname(fileURLToPath(import.meta.url)) + '/../end-user.happ';

const happDeveloperHapp =
	dirname(fileURLToPath(import.meta.url)) + '/../happ-developer.happ';

const infraProviderHapp =
	dirname(fileURLToPath(import.meta.url)) + '/../infra-provider.happ';

test('setup and send a push notification', async () => {
	await runScenario(async scenario => {
		const sender = await scenario.addPlayerWithApp({
			path: endUserHapp,
		});
		const recipient = await scenario.addPlayerWithApp({
			path: endUserHapp,
		});
		const serviceProvider = await scenario.addPlayerWithApp({
			path: serviceProviderHapp,
		});
		const happDevelop = await scenario.addPlayerWithApp({
			path: happDeveloperHapp,
		});
		const infraProvider = await scenario.addPlayerWithApp({
			path: infraProviderHapp,
		});

		await scenario.shareAllAgents();

		await infraProvider.namedCells
			.get('push_notifications_service_providers_manager')
			.callZome({
				zome_name: 'push_notifications_service_providers_manager',
				fn_name: 'enable_service',
				payload: {},
			});

		await dhtSync(
			[infraProvider, serviceProvider],
			infraProvider.cells[0].cell_id[0],
		);
	});
});
