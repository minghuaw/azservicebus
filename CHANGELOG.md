# CHANGELOG

## 0.25.0

1. Updated dependencies
   1. `azure_core` to "0.25"
   2. `azure_identity` to "0.25"

2. Updated ServiceBusClient to accept Arc-wrapped credentials to match azure_identity's new default credential type.

## 0.21.0

1. Updated dependencies
   1. `azure_core` to "0.21"
   2. `azure_identity` to "0.21"
   3. `fe2o3-amqp-types` to "0.14"
   4. `fe2o3-amqp` to "0.14"
   5. `fe2o3-amqp-cbs` to "0.14"
   6. `fe2o3-amqp-management` to "0.14"
   7. `fe2o3-amqp-ws` to "0.14"
   8. `serde_amqp` to "0.14"


## 0.20.4

1. Added support for unsecured connection. This is useful for testing with Azure Service Bus
   emulator.

## 0.20.3

1. Added `ServiceBusMessage::application_properties_mut()`

## 0.20.2

1. Fixed a bug where the message state cannot be converted from types other than i64
2. Updated readme

## 0.20.1

1. Fixed the bug that peek message response with a status code of 204 is not handled correctly.
   [#14](https://github.com/minghuaw/azservicebus/issues/14),
   [#15](https://github.com/minghuaw/azservicebus/issues/15)
2. Exposed `ServiceBusSessionReceiver::session_locked_until()`
3. Reworked internal error propagation of control link attachment

## 0.20.0

1. Updated dependencies
   1. `base64` to "0.22"
   2. `indexmap` to "2"
   3. `azure_core` to "0.20"
   4. `fe2o3-amqp` to "0.10"
   5. `fe2o3-amqp-types` to"0.10"
   6. `fe2o3-amqp` to "0.10"
   7. `fe2o3-amqp-cbs` to "0.10"
   8. `fe2o3-amqp-management` to "0.10"
   9. `fe2o3-amqp-ws` to "0.10"
   10. `serde_amqp` to "0.10"

## 0.19.2

- Updated dependencies
  - `fe2o3-amqp-types` to "0.7.2"
  - `fe2o3-amqp` to "0.8.27"
  - `fe2o3-amqp-management` to "0.2.3"
  - `serde_amqp` to "0.5.10"
- Use explicit `OrderedMap::swap_remove` instead of the deprecated `OrderedMap::remove`

## 0.19.1

- Removed `async_trait` and use async fn in trait feature stablized in 1.75.0.
- Set the minimum supported rust version to 1.75.0.

## 0.19.0

- Updated `azure_core` to "0.19.0"

## 0.18.0

- Updated `azure_core` to "0.18.0"
- Fixed a problem with retry mechanism that immediately fails if the connection recovery attempt
  fails

## 0.17.0

- Updated `azure_core` to "0.17.0"

## 0.16.0

- Migrated to separate [github repo](https://github.com/minghuaw/azservicebus)
- Updated `azure_core` dependency to 0.16.0

## 0.15.1 - Sep. 24, 2023

- Updated `fe2o3-amqp-ws` dependency to 0.4.0, which includes an upstream fix for [CVE-2023-43669](https://github.com/snapview/tungstenite-rs/pull/379).

## 0.15.0 - Sep. 15, 2023

- Updated `azure_core` dependency to 0.15.0
- Fixed clippy warnings

## 0.14.0 - Sep. 7, 2023

- Unified error type for most public functions to `azure_core::Error`
- Changed versioning to follow that of `azure_core`
- Fixed bug with `TokenCredential` support

## 0.1.2 - Feb. 7, 2023

- Added support for `wasm32-unknown-unknown` target

## 0.1.1 - Jan. 10, 2023

- Fixed typo in readme

## 0.1.0 - Jan. 10, 2023

- Initial release
