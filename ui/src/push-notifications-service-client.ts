import {
	ActionHash,
	AgentPubKey,
	AppClient,
	CreateLink,
	Delete,
	DeleteLink,
	EntryHash,
	Link,
	Record,
	SignedActionHashed,
} from '@holochain/client';
import { EntryRecord, ZomeClient } from '@darksoil-studio/holochain-utils';

import { PushNotificationsServiceSignal, ServiceAccountKey } from './types.js';

export class PushNotificationsServiceClient extends ZomeClient<PushNotificationsServiceSignal> {
	constructor(
		public client: AppClient,
		public roleName: string,
		public zomeName = 'push_notifications_service',
	) {
		super(client, roleName, zomeName);
	}

	publishServiceAccountKey(
		fcmProjectId: string,
		serviceAccountKey: ServiceAccountKey,
	): Promise<void> {
		return this.callZome('publish_service_account_key', {
			fcm_project_id: fcmProjectId,
			service_account_key: serviceAccountKey,
		});
	}

	async deleteFcmProject(fcmProjectId: string) {
		await this.callZome('delete_fcm_project', fcmProjectId);
	}

	getAllFcmProjects(): Promise<Array<string>> {
		return this.callZome('get_all_fcm_projects', undefined);
	}

	getCurrentServiceAccountKey(
		fcmProject: string,
	): Promise<ServiceAccountKey | undefined> {
		return this.callZome('get_current_service_account_key', fcmProject);
	}
}
