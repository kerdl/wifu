# Config documentation
If you want a little more from this app, you can adjust the config file, referring to this documentation.


# `ping`
This section is responsible for ping config.
##### Default:
```json
"ping": {
  "domains": {
    "list": [
      "google.com",
      "amazon.com",
      "microsoft.com"
    ],
    "mode": "firstIpFromEach"
  },
  "timeoutMs": 1500,
  "intervalMs": 1000,
  "maxErrors": 3
}
```

## `ping.domains`
Specifies domain ping settings:
- Which domains to ping
- The DNS lookup mode

## `ping.domains.list`
##### Default: `["google.com", "amazon.com", "microsoft.com"],`

Contains domains to ping.

Since the app's functionality is limited, there are some restrictions:
- Do not put protocols at the beginning (i.e. `https://google.com`)
- Do not put IP addresses (i.e. `209.85.233.113`)
- Do not specify ports (i.e. `google.com:443`)

There is no limit of domains, you can put as much as you want.

## `ping.domain.mode`
##### Default: `firstIpFromEach`
Specifies the DNS lookup way for the domains.
Each domain can have multiple IPs.

Possible values:
| Value           | Explanation                                                                            |
|-----------------|----------------------------------------------------------------------------------------|
| `firstIpFromEach` | Take the first IP for each domain.                                                     |
| `allIpsFromEach`  | Take all available IPs of each domain. This can result in 3-4 IPs for a single domain. |

## `ping.timeoutMs`
##### Default: `1500` (1.5 secs)
The maximum wait time for one ping, in milliseconds.
If this amount is exceeded, the app will start pinging other IP.

## `ping.intervalMs`
##### Default: `1000` (1 sec)
How much time to sleep between pings, in milliseconds.

## `ping.maxErrors`
##### Default: `3` (3 pings)
How much times ping can fail continuously before triggering a WI-FI swicth.
Every time pinging succeeds, the counter resets.

# `interfaces`
This section is responsible for the wireless interface config.
##### Default:
```json
"interfaces": {
  "priority": []
}
```


## `interfaces.priority`
##### Default: `[]`
Controls the priority of wireless interfaces connected to the machine. Handy if you have more than 1 connected.

This list is formed by interfaces GUID's.

Leave this list empty if you don't want the app to choose an interface that you may plug in while the app is running.

##### Example:
```json
"interfaces": {
  "priority": [
    "0A47A98D-B27B-4196-92BF-49E243BE8201",
    "B99E0C20-E4F5-44D3-B6C0-0ABAEACC0C7D"
  ]
}
```
In this example, `0A...` will always be a #1 choice, even if there's some other one already connected and working properly.

To put that into perspective, imagine that `B9...` or something else is currently chosen, because `0A...` is not connected. The moment you plug in `0A...`, this app will immediately choose `0A...`, unchoosing the `B9...` one and disconnecting the network from it. **Yes, this will produce downtime**. The network will be automatically connected on `0A...` in a few seconds.

##### How do you get the GUIDs?
Run the app and connect/disconnect your wireless interfaces. If they are **Plug&Play (USB adapters)**, you can just eject/insert them. If not, disable them in `Device Manager`. This will print their GUIDs in the console.
```
- INTERFACE: DISCONNECTED "TP-Link Wireless USB Adapter" (GUID 0A47A98D-B27B-4196-92BF-49E243BE8201)
+ INTERFACE: CONNECTED "D-Link DWA-131 Wireless N Nano USB Adapter(rev.E)" (GUID B99E0C20-E4F5-44D3-B6C0-0ABAEACC0C7D)
```


# `wifi`
This section is responsible for the WI-FI networks you want to use with this app.
##### Default:
```json
"wifi": {
  "networks": [],
  "scan": {
    "timeoutMs": 3000
  },
  "connect": {
    "timeoutMs": 4000
  },
  "disconnect": {
    "timeoutMs": 4000
  }
}
```

## `wifi.networks`
##### Default: `[]`
Holds the WI-FI networks that the app will use.

The list is also a priority. The moment you start the app, the **1st** network in the list would be the **1st** choice. However, unlike interface priority, if currenly chosen network is **2nd**, it won't go to the **1st** network even if it's available and works fine.

If your networks are encrypted, make sure to put the correct password before you run the program, otherwise you may have to either:
- Delete this network's profile `(Settings -> Network & Internet -> Wi-Fi -> Manage known networks)`
- Connect to it manually in Windows GUI, typing in the correct password, so Windows overwrites the old incorrect config.

And finally, put the correct password in the config file.

*Yes, that sucks. App's still really limited and buggy.*

##### Example:
```json
"networks": [
  {
    "ssid": "Home Wifi",
    "password": "amogUSSR",
  },
  {
    "ssid": "Unprotected WiFi",
  },
  {
    "ssid": "Unprotected WiFi 2",
    "password": null
  }
]
```

In this example, the **#1** priority is `Home Wifi`. It will be chosen the moment you start the app. If this network fails, it'll switch to the **#2** network `Unprotected WiFi` and so on.

Switching is performed in a looping manner, meaning if **#3** `Unprotected WiFi 2` fails, it'll go back to **#1** `Home Wifi`.

## `wifi.scan`
This section is responsible for SSID scanning options.

## `wifi.scan.timeoutMs`
##### Default: `3000` (3 secs)

The maximum time a scan process can go, in milliseconds. If that amount is exceeded, it's instantly cancelled.

## `wifi.connect`
This section is responsible for WI-FI connection options.

## `wifi.connect.timeoutMs`
##### Default: `4000` (4 secs)

The maximum time to wait for a connection to specified network, in milliseconds. If that amount is exceeded, it's instantly cancelled.

## `wifi.disconnect`
This section is responsible for WI-FI disconnection options.

## `wifi.disconnect.timeoutMs`
##### Default: `4000` (4 secs)

The maximum time to wait for a disconnection from specified network, in milliseconds. If that amount is exceeded, it's instantly cancelled.