import { createContext } from '@lit/context';

import { PushNotificationsServiceStore } from './push-notifications-service-store.js';

export const pushNotificationsServiceStoreContext =
	createContext<PushNotificationsServiceStore>(
		'push_notifications_service/store',
	);
