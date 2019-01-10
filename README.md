# Purpose

This is a crude REST geocode proxy service. It consists of a command-line application starting a HTTP server providing a REST API to geocode addresses. Geocoding consists in finding the latitude and longitude of an address or part of an address.

Note that this service does not geocode by itself and merely forwards requests to other online services. It currently delegates to the following services:

- [Here](https://developer.here.com/documentation/geocoder/topics/quick-start-geocode.html)
- [MapQuest](https://developer.mapquest.com/documentation/geocoding-api/)

# Building

The following tools need to be installed:

- [git](https://git-scm.com/) [2.16.2 or newer]
- [Rust](https://www.rust-lang.org/tools/install) [1.31.1 or newer]

Clone the repository:

```sh
git clone https://github.com/stephaneyfx/geocode-rs.git
```

**Note:** An internet connection is required to download dependencies during the build.

In a shell, navigate to the cloned repository and run the following:

```sh
cargo build --release
```

# Configuration

The proxy service needs some keys to use the services it delegates the geocoding to. These keys must be provided through a configuration file with the following format:

```json
{
    "sock_addr": "127.0.0.1:8080",
    "finder": {
        "protocols": [
            {
                "Here": {
                    "app_id": "...",
                    "app_code": "..."
                }
            },
            {
                "MapQuest": {
                    "key": "..."
                }
            }
        ]
    }
}
```

The `protocols` array can contain either or both backend services.

# Running the service

To start the service, execute the following (arguments to the service are given after "--"):

```sh
cargo run --release -- --config /path/to/config.json
```

Help is available by running:

```sh
cargo run --release -- --help
```

# REST API
## Geocode

### Request
`http://hostname/geocode?location=350+w+georgia+st,+Vancouver`

### Response
#### Success
```json
{
  "Ok": {
    "latitude": 49.2801599,
    "longitude": -123.1147572
  }
}
```

#### Error
```json
{
  "Err": {
    "kind": "BadRequest",
    "cause": []
  }
}
```

# Additional functionality

The service also allows to automatically open Google Maps with the coordinates it gets.
`http://hostname/map?location=350+w+georgia+st,+Vancouver`

# Supported platforms

Tested on Windows 10 and ArchLinux.
