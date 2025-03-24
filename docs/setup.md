# Setup

> [!WARNING]
> This guide assumes that you have scaffolded a hApp with the [TNESH stack template](https://darksoil.studio/tnesh-stack).

1. Run this to scaffold this zome in your hApp:

```bash
nix run github:darksoil-studio/push-notifications-service-provider-zome#scaffold
```

This will do the following:
  - Add the `github:darksoil-studio/push-notifications-service-provider-zome` flake input in your `flake.nix`.
  - Add the `push_notifications_service_provider` coordinator and integrity zome packages to the `dna.nix` that you select.
  - Add the UI package for `@darksoil-studio/push-notifications-service-provider-zome` as a dependency of your UI package.
  - Add the `<push-notifications-service-provider-context>` element at the top level of your application.

That's it! You have now integrated the `push_notifications_service_provider` coordinator and integrity zomes and their UI into your app!

Now, [choose which elements you need](/elements/push-notifications-service-provider-context.md) and import them like this:

```js
import "@darksoil-studio/push-notifications-service-provider-zome/dist/elements/push-notifications-service-provider-context.js";
```

And then they are ready be used just like any other HTML tag. 

> [!NOTE]
> Importing the elements from the UI package will define them in the global `CustomElementsRegistry`, which makes them available to be used like any normal HTML tag. You can read more about custom elements [here](https://darksoil.studio/tnesh-stack/guides/custom-elements).

# Example

You can see a full working example of the UI working in [here](https://github.com/darksoil-studio/push-notifications-service-provider-zome/blob/main/ui/demo/index.html).

