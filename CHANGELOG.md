# Changelog

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## Unreleased

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
