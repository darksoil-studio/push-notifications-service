
## hApp-care setup

```mermaid
sequenceDiagram

box grey HappDeveloper
    participant HDOC as OrganizationCell
    participant HDHC as HappCareCell
end

box grey DarksoilStudio
    participant DSOC as OrganizationCell
    participant DSHC as HappCareCell
    participant DSSPMC as ServiceProviderManagerCell
end

HDOC->>DSOC: create_happ()
DSOC->>DSOC: clone cell()


```

## hApp-care enabling push notifications

Note: each happ has its own PushNotificationsService dna where all the tokens for the users and the services account keys are stored.

```mermaid
sequenceDiagram

participant FCM

box grey HappDeveloper
    participant HDHC as HappCareCell
end

box grey DarksoilStudio
    participant DSHC as HappCareCell
    participant DSSPMC as ServiceProviderManagerCell
end

box grey PushNotificationsServiceProvider
    participant PNSPSPMC as ServiceProviderManagerCell
    participant PNSC as PushNotificationsServiceCell
    participant PNSPSC as HappCareServiceCell
    participant PNSP as ServiceProvider
end

HDHC->>FCM: get service account key
FCM->>HDHC: service account key
HDHC->>DSHC: enable push notifications(service account key)
DSHC->>DSSPMC: enable push notifications for happ(service account key)
DSSPMC->>PNSPSPMC: enable push notifications for happ(service account key)
PNSPSPMC->>PNSPSPMC: clone PushNotificiationsServiceCell(service account key)
PNSPSPMC->>PNSC: publish service account key
PNSC->>PNSPSC: announce as provider
```

## Device setup

```mermaid
sequenceDiagram

participant HappDeveloper

box grey Alice
    participant AliceApp as App
    participant ANP as NotificationPlugin
    participant AHSC as HappCareServiceCell
end

participant FCM

box grey PushNotificationsServiceProvider
    participant PNSPSC as HappCareServiceCell
    participant PNSC as PushNotificationsServiceCell
    participant PNSP as ServiceProvider
end

HappDeveloper->>AliceApp: google services api key

AliceApp->>ANP: google services api key
ANP->>FCM: get fcm token
ANP->>AliceApp: new_fcm_token
AliceApp->>AHSC: request_register_new_fcm_token
AHSC->>PNSPSC: register_new_fcm_token
PNSPSC->>PNSC: register_new_fcm_token

```

## Sending a notification


```mermaid
sequenceDiagram

box grey Bob
    participant BobApp as App
    participant BNP as NotificationPlugin
    participant BHSC as HappCareServiceCell
end

box grey PushNotificationsServiceProvider
    participant PNSPSC as HappCareServiceCell
    participant PNSC as PushNotificationsServiceCell
    participant PNSP as ServiceProvider
end

participant FCM

box grey Alice
    participant AliceApp as App
    participant AliceHapp as Happ
    participant ANP as NotificationPlugin
end

BobApp->>BHSC: send push notification
BHSC->>PNSPSC: send push notification(alice_pub_key)
PNSPSC->>PNSC: send push notification(alice_pub_key)
PNSC->>PNSC: get fcm token(alice_pub_key)
PNSC->>PNSP: send push notification(fcm token)

PNSP->>FCM: send push notification(fcm token)
FCM->>AliceApp: receive push notification
AliceApp->>AliceHapp: get notification content
AliceApp->>ANP: show notification

```