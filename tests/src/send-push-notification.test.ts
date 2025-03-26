import {
	ActionHash,
	Delete,
	DnaModifiers,
	Record,
	SignedActionHashed,
	encodeHashToBase64,
} from '@holochain/client';
import { Player, Scenario, dhtSync, runScenario } from '@holochain/tryorama';
import { decode, encode } from '@msgpack/msgpack';
import { toPromise } from '@tnesh-stack/signals';
import { EntryRecord } from '@tnesh-stack/utils';
import { cleanNodeDecoding } from '@tnesh-stack/utils/dist/clean-node-decoding.js';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { assert, test } from 'vitest';

import {
	endUserHapp,
	happDeveloperHapp,
	infraProviderHapp,
	serviceProviderHapp,
	setupInfraProvider,
} from './setup.js';

test('setup and send a push notification', async () => {
	await runScenario(async scenario => {
		const network_seed = `${Math.random()}`;
		const infraProvider = await setupInfraProvider(scenario);

		const serviceProvider = await scenario.addPlayerWithApp(
			{
				path: serviceProviderHapp,
			},
			{
				rolesSettings: {
					push_notifications_service_providers_manager: {
						type: 'Provisioned',
						modifiers: {
							properties: {
								progenitors: [encodeHashToBase64(infraProvider.agentPubKey)],
							},
						},
					},
					service_providers: {
						type: 'Provisioned',
						modifiers: {
							properties: {
								progenitors: [encodeHashToBase64(infraProvider.agentPubKey)],
							},
						},
					},
				},
			},
		);

		const happDeveloper = await scenario.addPlayerWithApp(
			{
				path: happDeveloperHapp,
			},
			{
				rolesSettings: {
					service_providers: {
						type: 'Provisioned',
						modifiers: {
							properties: {
								progenitors: [encodeHashToBase64(infraProvider.agentPubKey)],
							},
						},
					},
				},
			},
		);
		const sender = await scenario.addPlayerWithApp(
			{
				path: endUserHapp,
			},
			{
				rolesSettings: {
					service_providers: {
						type: 'Provisioned',
						modifiers: {
							properties: {
								progenitors: [encodeHashToBase64(infraProvider.agentPubKey)],
							},
						},
					},
				},
			},
		);
		const recipient = await scenario.addPlayerWithApp(
			{
				path: endUserHapp,
			},
			{
				rolesSettings: {
					service_providers: {
						type: 'Provisioned',
						modifiers: {
							properties: {
								progenitors: [encodeHashToBase64(infraProvider.agentPubKey)],
							},
						},
					},
				},
			},
		);

		await scenario.shareAllAgents();

		await infraProvider.namedCells
			.get('push_notifications_service_providers_manager')
			.callZome({
				zome_name: 'push_notifications_service_providers_manager',
				fn_name: 'create_clone_service_request',
				payload: {
					dna_modifiers: {
						properties: encode({}),
						network_seed,
						origin_time: new Date().valueOf() * 1000,
						quantum_time: {
							nanos: 0,
							secs: 1,
						},
					} as DnaModifiers,
				},
			});

		await dhtSync(
			[infraProvider, serviceProvider],
			infraProvider.cells[0].cell_id[0],
		);
	});
});
