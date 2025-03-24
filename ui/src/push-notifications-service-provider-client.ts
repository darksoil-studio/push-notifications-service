import { 
  SignedActionHashed,
  CreateLink,
  Link,
  DeleteLink,
  Delete,
  AppClient, 
  Record, 
  ActionHash, 
  EntryHash, 
  AgentPubKey,
} from '@holochain/client';
import { EntryRecord, ZomeClient } from '@tnesh-stack/utils';

import { PushNotificationsServiceProviderSignal } from './types.js';

export class PushNotificationsServiceProviderClient extends ZomeClient<PushNotificationsServiceProviderSignal> {

  constructor(public client: AppClient, public roleName: string, public zomeName = 'push_notifications_service_provider') {
    super(client, roleName, zomeName);
  }
}
