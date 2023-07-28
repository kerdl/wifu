# Automatic WI-FI switcher

[Config documentation](https://github.com/kerdl/wifu/blob/master/CFG-DOC.md)

This is a Windows-only tool for automatic WI-FI switching.
Basically, its functionality can be broken down into 3 steps:
- Connect to the WI-FI network specified in the config
- Ping websites with the most uptime (google.com, amazon.com, microsoft.com by default)
- Switch WI-FI to the other one in the config, if it fails to reach the websites

## Why

I have a weird infrastructure at work, where the 
main Windows server connects to a wireless modem by WI-FI
and shares that internet by LAN with everyone else.

If currently working modem decides to fail,
the only way to recover is to manually switch
to other's modem WI-FI.

This tool does that automatically, giving
much more uptime across the office,
even without my presence.

## Remarks
This app **WILL NOT** be frequently maintained and improved, as it already serves the functionality I need. I will fix bugs if they are problematic to me, but no improvements. But I would love to accept your PR.

## How to run
0. Install Windows 10 or 11 if still didn't
1. Install memory-safe (!) [Rust compiler](https://www.rust-lang.org/tools/install)
2. Install [Microsoft Visual Studio and Windows SDK](https://visualstudio.microsoft.com/downloads/)
3. If you have [Git](https://git-scm.com/download/win), clone this repository, or download it from GitHub, if not
```bat
git clone https://github.com/kerdl/wifu
```
4. CD to it
```bat
cd wifu
```
5. Run
```bat
cargo run --release
```

First startup will produce the following output:
```
o The app was initialized and a config file was created here: .\wifu-data\cfg.json
! Now, open the config file and fill in the networks you want to use with this app
? "How to" instructions can be found here: https://github.com/kerdl/wifu
```

6. Now, open the config file located at the path. It should look like this:
```json
{
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
  },
  "interfaces": {
    "priority": []
  },
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
}
```

7. Add a couple of networks to the `wifi.networks` list:
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

The final config would look like this:
```json
{
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
  },
  "interfaces": {
    "priority": []
  },
  "wifi": {
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
    ],
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
}
```

8. Run again
```bat
cargo run --release
```

The full config documentation can be found [here](https://github.com/kerdl/wifu/blob/master/CFG-DOC.md)