import { AppClient } from '@holochain/client';
import { consume, provide } from '@lit/context';
import { localized, msg } from '@lit/localize';
import { mdiInformationOutline } from '@mdi/js';
import '@shoelace-style/shoelace/dist/components/alert/alert.js';
import '@shoelace-style/shoelace/dist/components/input/input.js';
import '@shoelace-style/shoelace/dist/components/spinner/spinner.js';
import {
	appClientContext,
	notify,
	notifyError,
	onSubmit,
	sharedStyles,
	wrapPathInSvg,
} from '@tnesh-stack/elements';
import '@tnesh-stack/elements/dist/elements/display-error.js';
import { SignalWatcher } from '@tnesh-stack/signals';
import { LitElement, css, html } from 'lit';
import { customElement, property, query, state } from 'lit/decorators.js';

import { pushNotificationsServiceStoreContext } from '../context.js';
import { PushNotificationsServiceStore } from '../push-notifications-service-store.js';
import { ServiceAccountKey } from '../types.js';

/**
 * @element add-fcm-project
 */
@localized()
@customElement('add-fcm-project')
export class AddFcmProject extends SignalWatcher(LitElement) {
	@consume({ context: pushNotificationsServiceStoreContext })
	@property({ type: Object })
	store!: PushNotificationsServiceStore;

	@state()
	serviceAccountKey: ServiceAccountKey | undefined;

	async addFcmProject(fields: any) {
		try {
			await this.store.client.publishServiceAccountKey(
				fields['fcm-project-id'],
				this.serviceAccountKey!,
			);
			this.dispatchEvent(
				new CustomEvent('fcm-project-added', {
					bubbles: true,
					composed: true,
				}),
			);
			notify(msg('FCM project added.'));
		} catch (e) {
			console.error(e);
			notifyError(msg('Failed to add FCM project.'));
		}
	}

	@query('input')
	private _serviceAccountFilePicker!: HTMLInputElement;

	async onServiceAccountKeyUploaded() {
		if (
			this._serviceAccountFilePicker.files &&
			this._serviceAccountFilePicker.files[0]
		) {
			const file = this._serviceAccountFilePicker.files[0];
			let reader = new FileReader();
			reader.onload = event => {
				const v = JSON.parse(event.target!.result as any);
				this.serviceAccountKey = {
					auth_provider_x509_cert_url: v.auth_provider_x509_cert_url,
					auth_uri: v.auth_uri,
					client_email: v.client_email,
					client_id: v.client_id,
					client_x509_cert_url: v.client_x509_cert_url,
					type: v.type,
					private_key: v.private_key,
					private_key_id: v.private_key_id,
					project_id: v.project_id,
					token_uri: v.token_uri,
				};
			};
			reader.readAsText(file);
		}
	}

	render() {
		return html` <form
			class="column"
			style="gap: 16px; flex: 1"
			${onSubmit(fields => this.addFcmProject(fields))}
		>
			<span class="title">${msg('Add FCM Project')}</span>
			<sl-input .label=${msg('FCM Project ID')} required name="fcm-project-id">
			</sl-input>

			<div class="column" style="gap: 8px">
				<span>${msg('Service Account Key File')}*</span>
				<input
					type="file"
					accept="application/JSON"
					@change=${this.onServiceAccountKeyUploaded}
				/>
			</div>

			<sl-button
				type="submit"
				.disabled=${!this.serviceAccountKey}
				variant="primary"
				>${msg('Add FCM Project')}
			</sl-button>
		</form>`;
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
