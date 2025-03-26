# `<all-clone-service-requests>`

## Usage

0. If you haven't already, [go through the setup for the module](/setup).

1. Import the `<all-clone-service-requests>` element somewhere in the javascript side of your web-app like this:

```js
import '@darksoil-studio/push-notifications-service-providers-manager/dist/elements/all-clone-service-requests.js'
```

2. Use it in the html side of your web-app like this:

```html
<all-clone-service-requests>
</all-clone-service-requests>
```

> [!WARNING]
> Like all the elements in this module, `<all-clone-service-requests>` needs to be placed inside an initialized `<push-notifications-service-providers-manager-context>`.

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

import { PushNotificationsServiceProvidersManagerZomeMock, sampleCloneServiceRequest } from "../../ui/src/mocks.ts";
import { PushNotificationsServiceProvidersManagerStore } from "../../ui/src/push-notifications-service-providers-manager-store.ts";
import { PushNotificationsServiceProvidersManagerClient } from "../../ui/src/push-notifications-service-providers-manager-client.ts";

onMounted(async () => {
  // Elements need to be imported on the client side, not the SSR side
  // Reference: https://vitepress.dev/guide/ssr-compat#importing-in-mounted-hook
  await import('@api-viewer/docs/lib/api-docs.js');
  await import('@api-viewer/demo/lib/api-demo.js');
  await import('@darksoil-studio/profiles-zome/dist/elements/profiles-context.js');
  if (!customElements.get('push-notifications-service-providers-manager-context')) await import('../../ui/src/elements/push-notifications-service-providers-manager-context.ts');
  if (!customElements.get('all-clone-service-requests')) await import('../../ui/src/elements/all-clone-service-requests.ts');

  const profiles = await demoProfiles();

  const profilesMock = new ProfilesZomeMock(
    profiles,
    Array.from(profiles.keys())[0]
  );
  const profilesStore = new ProfilesStore(new ProfilesClient(profilesMock, "push_notifications_service_providers_manager"));

  const mock = new PushNotificationsServiceProvidersManagerZomeMock();
  const client = new PushNotificationsServiceProvidersManagerClient(mock, "push_notifications_service_providers_manager");

  const cloneServiceRequest = await sampleCloneServiceRequest(client);

  const record = await mock.create_clone_service_request(cloneServiceRequest);

  const store = new PushNotificationsServiceProvidersManagerStore(client);
  
  render(html`
    <profiles-context .store=${profilesStore}>
      <push-notifications-service-providers-manager-context .store=${store}>
        <api-demo src="custom-elements.json" only="all-clone-service-requests" exclude-knobs="store">
          <template data-element="all-clone-service-requests" data-target="host">
            <all-clone-service-requests ></all-clone-service-requests>
          </template>
        </api-demo>
      </push-notifications-service-providers-manager-context>
    </profiles-context>
  `, document.querySelector('element-demo'))
  })


</script>

## API Reference

`<all-clone-service-requests>` is a [custom element](https://web.dev/articles/custom-elements-v1), which means that it can be used in any web app or website. Here is the reference for its API:

<api-docs src="custom-elements.json" only="all-clone-service-requests">
</api-docs>
