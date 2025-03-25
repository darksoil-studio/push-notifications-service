# `<service-account-key-summary>`

## Usage

0. If you haven't already, [go through the setup for the module](/setup).

1. Import the `<service-account-key-summary>` element somewhere in the javascript side of your web-app like this:

```js
import '@darksoil-studio/push-notifications-service/dist/elements/service-account-key-summary.js'
```

2. Use it in the html side of your web-app like this:

::: code-group
```html [Lit]
<service-account-key-summary .serviceAccountKeyHash=${ serviceAccountKeyHash }>
</service-account-key-summary>
```

```html [React]
<service-account-key-summary serviceAccountKeyHash={ serviceAccountKeyHash }>
</service-account-key-summary>
```

```html [Angular]
<service-account-key-summary [serviceAccountKeyHash]="serviceAccountKeyHash">
</service-account-key-summary>
```

```html [Vue]
<service-account-key-summary :serviceAccountKeyHash="serviceAccountKeyHash">
</service-account-key-summary>
```

```html [Svelte]
<service-account-key-summary service-account-key-hash={encodeHashToBase64(serviceAccountKeyHash)}>
</service-account-key-summary>
```
:::

> [!WARNING]
> Like all the elements in this module, `<service-account-key-summary>` needs to be placed inside an initialized `<push-notifications-service-context>`.

## Demo

Here is an interactive demo of the element:

<element-demo>
</element-demo>

<script setup>
import { onMounted } from "vue";
import { ProfilesClient, ProfilesStore } from '@darksoil-studio/profiles-zome';
import { demoProfiles, ProfilesZomeMock } from '@darksoil-studio/profiles-zome/dist/mocks.js';
import { decodeHashFromBase64, encodeHashToBase64 } from '@holochain/client';
import { render } from "lit";
import { html, unsafeStatic } from "lit/static-html.js";

import { PushNotificationsServiceZomeMock, sampleServiceAccountKey } from "../../ui/src/mocks.ts";
import { PushNotificationsServiceStore } from "../../ui/src/push-notifications-service-store.ts";
import { PushNotificationsServiceClient } from "../../ui/src/push-notifications-service-client.ts";

onMounted(async () => {
  // Elements need to be imported on the client side, not the SSR side
  // Reference: https://vitepress.dev/guide/ssr-compat#importing-in-mounted-hook
  await import('@api-viewer/docs/lib/api-docs.js');
  await import('@api-viewer/demo/lib/api-demo.js');
  await import('@darksoil-studio/profiles-zome/dist/elements/profiles-context.js');
  if (!customElements.get('push-notifications-service-context')) await import('../../ui/src/elements/push-notifications-service-context.ts');
  if (!customElements.get('service-account-key-summary')) await import('../../ui/src/elements/service-account-key-summary.ts');

  const profiles = await demoProfiles();

  const profilesMock = new ProfilesZomeMock(
    profiles,
    Array.from(profiles.keys())[0]
  );
  const profilesStore = new ProfilesStore(new ProfilesClient(profilesMock, "push_notifications_service"));

  const mock = new PushNotificationsServiceZomeMock();
  const client = new PushNotificationsServiceClient(mock, "push_notifications_service");

  const serviceAccountKey = await sampleServiceAccountKey(client);

  const record = await mock.create_service_account_key(serviceAccountKey);

  const store = new PushNotificationsServiceStore(client);
  
  render(html`
    <profiles-context .store=${profilesStore}>
      <push-notifications-service-context .store=${store}>
        <api-demo src="custom-elements.json" only="service-account-key-summary" exclude-knobs="store">
          <template data-element="service-account-key-summary" data-target="host">
            <service-account-key-summary serviceAccountKey-hash="${unsafeStatic(encodeHashToBase64(record.signed_action.hashed.hash))}"></service-account-key-summary>
          </template>
        </api-demo>
      </push-notifications-service-context>
    </profiles-context>
  `, document.querySelector('element-demo'))
  })


</script>

## API Reference

`<service-account-key-summary>` is a [custom element](https://web.dev/articles/custom-elements-v1), which means that it can be used in any web app or website. Here is the reference for its API:

<api-docs src="custom-elements.json" only="service-account-key-summary">
</api-docs>
