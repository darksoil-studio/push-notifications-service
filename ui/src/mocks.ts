import {
  AgentPubKeyMap,
  decodeEntry,
  fakeEntry,
  fakeCreateAction,
  fakeUpdateEntry,
  fakeDeleteEntry,
  fakeRecord,
  pickBy,
  ZomeMock,
  HoloHashMap,
  HashType,
  hash
} from "@darksoil-studio/holochain-utils";
import {
  decodeHashFromBase64,
  NewEntryAction,
  AgentPubKey,
  ActionHash,
  EntryHash,
  Delete,
  AppClient,
  fakeAgentPubKey,
  fakeDnaHash,
  Link,
  fakeActionHash,
  SignedActionHashed,
  fakeEntryHash,
  Record,
} from "@holochain/client";
import { PushNotificationsServiceProviderClient } from './push-notifications-service-provider-client.js'

export class PushNotificationsServiceProviderZomeMock extends ZomeMock implements AppClient {
  constructor(
    myPubKey?: AgentPubKey
  ) {
    super("push_notifications_service_provider_test", "push_notifications_service_provider", "push_notifications_service_provider_test_app", myPubKey);
  }
  
}
