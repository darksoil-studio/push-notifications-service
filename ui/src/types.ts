import {
	ActionHash,
	AgentPubKey,
	Create,
	CreateLink,
	Delete,
	DeleteLink,
	DnaHash,
	EntryHash,
	Record,
	SignedActionHashed,
	Update,
} from '@holochain/client';
import { ActionCommittedSignal } from '@darksoil-studio/holochain-utils';

export type PushNotificationsServiceSignal = ActionCommittedSignal<
	EntryTypes,
	LinkTypes
>;

export type EntryTypes = never;

export type LinkTypes = string;

export interface PublishServiceAccountKeyInput {
	fcm_project_id: string;
	service_account_key: ServiceAccountKey;
}

export interface ServiceAccountKey {
	type: string | undefined;
	project_id: string | undefined;
	private_key_id: string | undefined;
	private_key: string;
	client_email: string;
	client_id: string | undefined;
	auth_uri: string | undefined;
	token_uri: string;
	auth_provider_x509_cert_url: string | undefined;
	client_x509_cert_url: string | undefined;
}
