import { AppClient } from '@holochain/client';
import { consume, provide } from '@lit/context';
import { appClientContext } from '@tnesh-stack/elements';
import { LitElement, css, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';

import { pushNotificationsServiceStoreContext } from '../context.js';
import { PushNotificationsServiceClient } from '../push-notifications-service-client.js';
import { PushNotificationsServiceStore } from '../push-notifications-service-store.js';

/**
 * @element push-notifications-service-context
 */
@customElement('push-notifications-service-context')
export class PushNotificationsServiceContext extends LitElement {
	@consume({ context: appClientContext })
	private client!: AppClient;

	@provide({ context: pushNotificationsServiceStoreContext })
	@property({ type: Object })
	store!: PushNotificationsServiceStore;

	@property()
	role!: string;

	@property()
	zome = 'push_notifications_service';

	connectedCallback() {
		super.connectedCallback();
		if (this.store) return;
		if (!this.role) {
			throw new Error(
				`<push-notifications-service-context> must have a role="YOUR_DNA_ROLE" property, eg: <push-notifications-service-context role="role1">`,
			);
		}
		if (!this.client) {
			throw new Error(`<push-notifications-service-context> must either:
        a) be placed inside <app-client-context>
          or 
        b) receive an AppClient property (eg. <push-notifications-service-context .client=\${client}>) 
          or 
        c) receive a store property (eg. <push-notifications-service-context .store=\${store}>)
      `);
		}

		this.store = new PushNotificationsServiceStore(
			new PushNotificationsServiceClient(this.client, this.role, this.zome),
		);
	}

	render() {
		return html`<slot></slot>`;
	}

	static styles = css`
		:host {
			display: contents;
		}
	`;
}
