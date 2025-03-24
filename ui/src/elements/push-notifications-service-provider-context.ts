import { css, html, LitElement } from 'lit';
import { provide, consume } from '@lit/context';
import { customElement, property } from 'lit/decorators.js';
import { AppClient } from '@holochain/client';
import { appClientContext } from '@tnesh-stack/elements';

import { pushNotificationsServiceProviderStoreContext } from '../context.js';
import { PushNotificationsServiceProviderStore } from '../push-notifications-service-provider-store.js';
import { PushNotificationsServiceProviderClient } from '../push-notifications-service-provider-client.js';

/**
 * @element push-notifications-service-provider-context
 */
@customElement('push-notifications-service-provider-context')
export class PushNotificationsServiceProviderContext extends LitElement {
  @consume({ context: appClientContext })
  private client!: AppClient;

  @provide({ context: pushNotificationsServiceProviderStoreContext })
  @property({ type: Object })
  store!: PushNotificationsServiceProviderStore;

  @property()
  role!: string;

  @property()
  zome = 'push_notifications_service_provider';

  connectedCallback() {
    super.connectedCallback();
    if (this.store) return;
    if (!this.role) {
      throw new Error(`<push-notifications-service-provider-context> must have a role="YOUR_DNA_ROLE" property, eg: <push-notifications-service-provider-context role="role1">`);
    }
    if (!this.client) {
      throw new Error(`<push-notifications-service-provider-context> must either:
        a) be placed inside <app-client-context>
          or 
        b) receive an AppClient property (eg. <push-notifications-service-provider-context .client=\${client}>) 
          or 
        c) receive a store property (eg. <push-notifications-service-provider-context .store=\${store}>)
      `);
    }

    this.store = new PushNotificationsServiceProviderStore(new PushNotificationsServiceProviderClient(this.client, this.role, this.zome));
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

