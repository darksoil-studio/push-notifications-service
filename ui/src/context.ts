import { createContext } from '@lit/context';
import { PushNotificationsServiceProviderStore } from './push-notifications-service-provider-store.js';

export const pushNotificationsServiceProviderStoreContext = createContext<PushNotificationsServiceProviderStore>(
  'push_notifications_service_provider/store'
);

