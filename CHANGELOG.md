# Changelog

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## Unreleased

## 0.3.0

### added

- A `prelude` module.

### changed

- To fix name conflicts with `AddProtocolExt`, the respective types have changed names.
  - The `client` feature trait has changed to `AddClientProtocolExt`
  - The `server` feature trait has changed to `AddServerProtocolExt`
- Method names have changed:
  - `add_sendonly_protocol` has changed to `add_client_wo_protocol` and `add_server_wo_protocol`
  - `add_readonly_bounded_protocol` has changed to `add_client_ro_protocol` and `add_server_ro_protocol`
  - `add_readonly_unbounded_protocol` has changed to `add_client_ro_unbounded_protocol` and `add_server_ro_unbounded_protocol`
  - `add_bounded_protocol` has changed to `add_client_rw_protocol` and `add_server_rw_protocol`
  - `add_unbounded_protocol` has changed to `add_client_rw_unbounded_protocol` and `add_server_rw_unbounded_protocol`
- The `ConnectionRequest` event under the `client` feature has been renamed to `RtcClientRequestEvent`
- The `Payload` derive was renamed to `Protocol`
- Fields on `RtcClientState` and `RtcServerState` are now private, with accessor methods, e.g. `.id()` instead of `.id`

## 0.2.0

### added

- The `RtcClient` and `RtcServer` system parameters (prev. `NetworkReader`/`NetworkWriter`) have new methods:
  - `clear()` to clear all incoming messages in the buffer.

### changed

- To fix name conflicts with the `server` and `client` feature, the respective types have changed names.
  - `RtcState` has changed to `RtcClientState` or `RtcServerState`, depending on the feature.
  - `RtcStatus` has changed to `RtcClientStatus` or `RtcServerStatus`, depending on the feature.
  - `NetworkReader`/`NetworkWriter` have been both merged and changed to `RtcClient` or `RtcServer` respectively.
  - `RtcClient.read()` (previously `NetworkReader`) now returns a `Vec<_>` rather than a `Drain<'_, _>`.

## 0.1.1

### fixes

- Fixed blank README on crates.io

## 0.1.0

- Initial release. Crate was renamed to `bevy_rtc` from `bevy-rtc` and republished.
