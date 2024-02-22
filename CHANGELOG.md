# CHANGELOG

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
