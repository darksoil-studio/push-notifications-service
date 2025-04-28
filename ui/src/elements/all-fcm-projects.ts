import { AppClient } from '@holochain/client';
import { consume, provide } from '@lit/context';
import { localized, msg } from '@lit/localize';
import { mdiDelete, mdiInformationOutline } from '@mdi/js';
import '@shoelace-style/shoelace/dist/components/button/button.js';
import SlButton from '@shoelace-style/shoelace/dist/components/button/button.js';
import '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import SlDialog from '@shoelace-style/shoelace/dist/components/dialog/dialog.js';
import '@shoelace-style/shoelace/dist/components/spinner/spinner.js';
import {
	appClientContext,
	notify,
	notifyError,
	sharedStyles,
	wrapPathInSvg,
} from '@darksoil-studio/holochain-elements';
import '@darksoil-studio/holochain-elements/dist/elements/display-error.js';
import { SignalWatcher } from '@darksoil-studio/holochain-signals';
import { GetonlyMap } from '@darksoil-studio/holochain-utils';
import { LitElement, css, html } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { join } from 'lit/directives/join.js';

import { pushNotificationsServiceStoreContext } from '../context.js';
import { PushNotificationsServiceStore } from '../push-notifications-service-store.js';
import { ServiceAccountKey } from '../types.js';

/**
 * @element all-fcm-projects
 */
@localized()
@customElement('all-fcm-projects')
export class AllFcmProjects extends SignalWatcher(LitElement) {
	@consume({ context: pushNotificationsServiceStoreContext })
	@property({ type: Object })
	store!: PushNotificationsServiceStore;

	async addFcmProject(
		fcmProjectId: string,
		serviceAccountKey: ServiceAccountKey,
	) {
		try {
			await this.store.client.publishServiceAccountKey(
				fcmProjectId,
				serviceAccountKey,
			);
			notify(msg('Service account key published.'));
		} catch (e) {
			console.error(e);
			notifyError(msg('Failed to publish service account key.'));
		}
	}

	async onServiceAccountKeyUploaded(
		fcmProjectId: string,
		input: HTMLInputElement,
	) {
		if (input.files && input.files[0]) {
			const file = input.files[0];
			let reader = new FileReader();
			reader.onload = event => {
				const v = JSON.parse(event.target!.result as any);
				const serviceAccountKey: ServiceAccountKey = {
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
				this.addFcmProject(fcmProjectId, serviceAccountKey).finally(
					() => (input.value = ''),
				);
			};
			reader.readAsText(file);
		}
	}

	renderList(
		fcmProjectsWithServiceAccountKeys: ReadonlyMap<
			string,
			ServiceAccountKey | undefined
		>,
	) {
		if (fcmProjectsWithServiceAccountKeys.size === 0) {
			return html` <div
				class="column placeholder center-content"
				style="gap: 8px; flex: 1; padding: 32px"
			>
				<sl-icon
					.src=${wrapPathInSvg(mdiInformationOutline)}
					style="font-size: 64px;"
				></sl-icon>
				<span style="text-align: center">${msg('No FCM projects found.')}</span>
			</div>`;
		}

		return html`
			<div class="column" style="gap: 8px; flex: 1">
				${join(
					Array.from(fcmProjectsWithServiceAccountKeys.entries()).map(
						([fcmProjectId, serviceAccountKey]) =>
							html` <div class="row" style="gap: 16px; align-items: center">
								<span style="flex: 1">${fcmProjectId}</span>
								<sl-button
									@click=${() =>
										(
											this.shadowRoot!.getElementById(
												`${fcmProjectId}-dialog`,
											) as SlDialog
										).show()}
									>${msg('Service Account Key')}
								</sl-button>
								<sl-button
									circle
									outline
									variant="danger"
									@click=${() =>
										(
											this.shadowRoot!.getElementById(
												`${fcmProjectId}-delete-dialog`,
											) as SlDialog
										).show()}
									><sl-icon .src=${wrapPathInSvg(mdiDelete)}></sl-icon>
								</sl-button>
								<sl-dialog
									label=${msg('Service Account Key')}
									id="${fcmProjectId}-dialog"
								>
									<div class="column" style="gap: 32px">
										<pre style="overflow-x: auto">
${JSON.stringify(serviceAccountKey, null, 2)} </pre
										>

										<div class="column" style="gap: 16px">
											<span class="title"
												>${msg('Publish New Service Account Key')}</span
											>

											<input
												type="file"
												id="${fcmProjectId}-input"
												accept="application/JSON"
												@change=${() =>
													this.onServiceAccountKeyUploaded(
														fcmProjectId,
														this.shadowRoot!.getElementById(
															`${fcmProjectId}-input`,
														) as HTMLInputElement,
													)}
											/>
										</div>
									</div>
								</sl-dialog>
								<sl-dialog
									label=${msg('Delete FCM Project')}
									id="${fcmProjectId}-delete-dialog"
								>
									<div class="column" style="gap: 16px">
										<span
											>${msg('Are you sure you want to delete this project?')}
										</span>
										<span
											>${msg(
												'This will prevent push notifications from working for apps linked with this project.',
											)}
										</span>
									</div>
									<sl-button
										slot="footer"
										@click=${() =>
											(
												this.shadowRoot!.getElementById(
													`${fcmProjectId}-delete-dialog`,
												) as SlDialog
											).hide()}
										>${msg('Cancel')}
									</sl-button>
									<sl-button
										slot="footer"
										variant="danger"
										@click=${async (e: CustomEvent) => {
											const button = e.target as SlButton;
											button.loading = true;
											try {
												await this.store.client.deleteFcmProject(fcmProjectId);
												notify(msg('FCM project deleted.'));
											} catch (e) {
												console.error(e);
												notifyError(msg('Failed to delete the FCM project.'));
											}
											button.loading = false;
										}}
										>${msg('Delete FCM Project')}
									</sl-button>
								</sl-dialog>
							</div>`,
					),
					html`<sl-divider></sl-divider>`,
				)}
			</div>
		`;
	}

	render() {
		const projects = this.store.fcmProjectsServiceAccountKeys.get();

		switch (projects.status) {
			case 'pending':
				return html`<div
					style="display: flex; flex-direction: column; align-items: center; justify-content: center; flex: 1;"
				>
					<sl-spinner style="font-size: 2rem;"></sl-spinner>
				</div>`;
			case 'error':
				return html`<display-error
					.headline=${msg('Error fetching the FCM projects.')}
					.error=${projects.error}
				></display-error>`;
			case 'completed':
				return this.renderList(projects.value);
		}
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
