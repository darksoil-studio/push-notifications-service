import {
	AppWebsocket,
	DnaModifiers,
	Link,
	encodeHashToBase64,
} from '@holochain/client';
import { Player, Scenario, dhtSync, runScenario } from '@holochain/tryorama';
import { decode, encode } from '@msgpack/msgpack';
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

		const properties = {
			progenitors: [encodeHashToBase64(infraProvider.agentPubKey)],
		};

		const serviceProvider = await scenario.addPlayerWithApp(
			{
				path: serviceProviderHapp,
			},
			{
				rolesSettings: {
					push_notifications_service_providers_manager: {
						type: 'Provisioned',
						modifiers: {
							properties,
						},
					},
					service_providers: {
						type: 'Provisioned',
						modifiers: {
							properties,
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
							properties,
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
							properties,
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
							properties,
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
						properties: encode(properties),
						network_seed,
						origin_time: new Date().valueOf() * 1000,
						quantum_time: {
							nanos: 0,
							secs: 60 * 5,
						},
					} as DnaModifiers,
				},
			});

		await dhtSync(
			[infraProvider, serviceProvider],
			infraProvider.cells[0].cell_id[0],
		);

		const cloneServicesRequests: Array<CloneServiceRequest> =
			await serviceProvider.namedCells
				.get('push_notifications_service_providers_manager')
				.callZome({
					zome_name: 'push_notifications_service_providers_manager',
					fn_name: 'get_all_clone_service_requests',
					payload: undefined,
				});

		assert.equal(cloneServicesRequests.length, 1);

		await (serviceProvider.appWs as AppWebsocket).createCloneCell({
			modifiers: {
				...cloneServicesRequests[0].dna_modifiers,
				properties: decode(cloneServicesRequests[0].dna_modifiers.properties),
			},
			role_name: 'push_notifications_service',
		});

		await (serviceProvider.appWs as AppWebsocket).createCloneCell({
			modifiers: {
				...cloneServicesRequests[0].dna_modifiers,
				properties: decode(cloneServicesRequests[0].dna_modifiers.properties),
			},
			role_name: 'service_providers',
		});
	});
});

export interface CloneServiceRequest {
	dna_modifiers: DnaModifiers;
}
