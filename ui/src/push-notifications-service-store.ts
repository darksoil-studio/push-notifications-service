import { AsyncComputed, joinAsyncMap } from '@darksoil-studio/holochain-signals';
import { MemoMap, mapValues, retype, slice } from '@darksoil-studio/holochain-utils';

import { PushNotificationsServiceClient } from './push-notifications-service-client.js';
import {
	joinAsyncNormalMap,
	lazyLoadAndPollOrEvent,
	mapValuesNormalMap,
	sliceNormalMap,
} from './utils.js';

export class PushNotificationsServiceStore {
	constructor(public client: PushNotificationsServiceClient) {}

	fcmProjects = lazyLoadAndPollOrEvent(
		() => this.client.getAllFcmProjects(),
		10_000,
		refetch => this.client.onSignal(signal => refetch()),
	);

	serviceAccountKeys = new MemoMap((fcmProject: string) =>
		lazyLoadAndPollOrEvent(
			() => this.client.getCurrentServiceAccountKey(fcmProject),
			10_000,
			refetch => this.client.onSignal(signal => refetch()),
		),
	);

	fcmProjectsServiceAccountKeys = new AsyncComputed(() => {
		const projects = this.fcmProjects.get();
		if (projects.status !== 'completed') return projects;

		return joinAsyncNormalMap(
			mapValuesNormalMap(
				sliceNormalMap(this.serviceAccountKeys, projects.value),
				v => v.get(),
			),
		);
	});
}
