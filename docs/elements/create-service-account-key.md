# `<create-service-account-key>`

## Usage

0. If you haven't already, [go through the setup for the module](/setup).

1. Import the `<create-service-account-key>` element somewhere in the javascript side of your web-app like this:

```js
import '@darksoil-studio/push-notifications-service/dist/elements/create-service-account-key.js'
```

2. Use it in the html side of your web-app like this:


::: code-group
```html [Lit]
<create-service-account-key 
  .a=${ a }
>
</create-service-account-key>
```

```html [React]
<create-service-account-key
  a={ a }
>
</create-service-account-key>
```

```html [Angular]
<create-service-account-key
  [a]="a"
>
</create-service-account-key>
```

```html [Vue]
<create-service-account-key
  :a="a"
>
</create-service-account-key>
```

```html [Svelte]
<create-service-account-key
  a={encodeHashToBase64(a)}
>
</create-service-account-key>
```
:::

> [!WARNING]
> Like all the elements in this module, `<create-service-account-key>` needs to be placed inside an initialized `<push-notifications-service-context>`.

## Demo

Here is an interactive demo of the element:

<element-demo>
</element-demo>

<script setup>
import { onMounted } from "vue";
import { ProfilesClient, ProfilesStore } from '@darksoil-studio/profiles-zome';
import { demoProfiles, ProfilesZomeMock } from '@darksoil-studio/profiles-zome/dist/mocks.js';
import { decodeHashFromBase64 } from '@holochain/client';
import { render, html } from "lit";

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
  if (!customElements.get('create-service-account-key')) await import('../../ui/src/elements/create-service-account-key.ts');

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
        <api-demo src="custom-elements.json" only="create-service-account-key" exclude-knobs="store">
        </api-demo>
      </push-notifications-service-context>
    </profiles-context>
  `, document.querySelector('element-demo'))
  })


</script>

## API Reference

`<create-service-account-key>` is a [custom element](https://web.dev/articles/custom-elements-v1), which means that it can be used in any web app or website. Here is the reference for its API:

<api-docs src="custom-elements.json" only="create-service-account-key">
</api-docs>
