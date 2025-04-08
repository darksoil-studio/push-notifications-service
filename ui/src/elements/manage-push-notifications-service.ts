import { AppClient } from '@holochain/client';
import { consume, provide } from '@lit/context';
import { localized, msg } from '@lit/localize';
import { mdiInformationOutline, mdiPlus } from '@mdi/js';
import '@shoelace-style/shoelace/dist/components/card/card.js';
import '@shoelace-style/shoelace/dist/components/spinner/spinner.js';
import {
	appClientContext,
	sharedStyles,
	wrapPathInSvg,
} from '@tnesh-stack/elements';
import '@tnesh-stack/elements/dist/elements/display-error.js';
import { SignalWatcher } from '@tnesh-stack/signals';
import { LitElement, css, html } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';

import { pushNotificationsServiceStoreContext } from '../context.js';
import { PushNotificationsServiceStore } from '../push-notifications-service-store.js';
import './add-fcm-project.js';
import './all-fcm-projects.js';

/**
 * @element manage-push-notifications-service
 */
@localized()
@customElement('manage-push-notifications-service')
export class ManagePushNotifications extends SignalWatcher(LitElement) {
	@consume({ context: pushNotificationsServiceStoreContext })
	@property({ type: Object })
	store!: PushNotificationsServiceStore;

	@state()
	addingProject = false;

	render() {
		if (this.addingProject)
			return html`
				<div
					class="column"
					style="justify-content: center; align-items: center; flex: 1"
				>
					<sl-card>
						<add-fcm-project
							@fcm-project-added=${() => (this.addingProject = false)}
						></add-fcm-project>
					</sl-card>
				</div>
			`;

		return html` <div class="column" style="gap: 16px; flex: 1">
			<div class="row">
				<span style="flex: 1"> </span>
				<sl-button
					variant="primary"
					@click=${() => (this.addingProject = true)}
				>
					<sl-icon slot="prefix" .src=${wrapPathInSvg(mdiPlus)}></sl-icon>

					${msg('Add FCM project')}
				</sl-button>
			</div>
			<sl-card>
				<div class="column" style="gap: 16px; flex: 1">
					<span class="title">${msg('FCM Projects')}</span>
					<all-fcm-projects style="flex: 1"></all-fcm-projects>
				</div>
			</sl-card>
		</div>`;
	}

	static styles = [
		sharedStyles,
		css`
			:host {
				display: flex;
			}
		`,
	];
}
