import {
	ActionHash,
	AgentPubKey,
	AppBundleSource,
	AppCallZomeRequest,
	AppWebsocket,
	EntryHash,
	NewEntryAction,
	Record,
	RoleSettingsMap,
	encodeHashToBase64,
	fakeActionHash,
	fakeAgentPubKey,
	fakeDnaHash,
	fakeEntryHash,
} from '@holochain/client';
import {
	AgentApp,
	Player,
	Scenario,
	dhtSync,
	enableAndGetAgentApp,
	pause,
} from '@holochain/tryorama';
import { encode } from '@msgpack/msgpack';
import { EntryRecord } from '@tnesh-stack/utils';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

import { PushNotificationsServiceProviderClient } from '../../ui/src/push-notifications-service-provider-client.js';
import { PushNotificationsServiceProviderStore } from '../../ui/src/push-notifications-service-provider-store.js';

export const serviceProviderHapp =
	dirname(fileURLToPath(import.meta.url)) +
	'/../../workdir/push-notifications-service-provider.happ';

export const endUserHapp =
	dirname(fileURLToPath(import.meta.url)) + '/../end-user.happ';

export const happDeveloperHapp =
	dirname(fileURLToPath(import.meta.url)) + '/../happ-developer.happ';

export const infraProviderHapp =
	dirname(fileURLToPath(import.meta.url)) + '/../infra-provider.happ';

export async function setupInfraProvider(scenario: Scenario): Promise<Player> {
	const infraProviderConductor = await scenario.addConductor();

	const infraProviderPubKey = await infraProviderConductor
		.adminWs()
		.generateAgentPubKey();

	const rolesSettings: RoleSettingsMap = {
		push_notifications_service_providers_manager: {
			type: 'Provisioned',
			modifiers: {
				properties: {
					progenitors: [encodeHashToBase64(infraProviderPubKey)],
				},
			},
		},
	};

	const appInfo = await infraProviderConductor.installApp(
		{ path: infraProviderHapp },
		{
			agentPubKey: infraProviderPubKey,
			networkSeed: scenario.networkSeed,
			rolesSettings,
		},
	);

	const port = await infraProviderConductor.attachAppInterface();

	const issued = await infraProviderConductor
		.adminWs()
		.issueAppAuthenticationToken({
			installed_app_id: appInfo.installed_app_id,
		});
	const appWs = await infraProviderConductor.connectAppWs(issued.token, port);

	const infraProvider: AgentApp = await enableAndGetAgentApp(
		infraProviderConductor.adminWs(),
		appWs,
		appInfo,
	);
	return { conductor: infraProviderConductor, appWs, ...infraProvider };
}

export async function waitUntil(
	condition: () => Promise<boolean>,
	timeout: number,
) {
	const start = Date.now();
	const isDone = await condition();
	if (isDone) return;
	if (timeout <= 0) throw new Error('timeout');
	await pause(1000);
	return waitUntil(condition, timeout - (Date.now() - start));
}
