# Changelog

This changelog follows the patterns described here: <https://keepachangelog.com/en/1.0.0/>.

Subheadings to categorize changes are `added, changed, deprecated, removed, fixed, security`.

## 0.8.10

### added

- `.add_sendonly_protocol()` for protocols that are never read.
- `.add_readonly_bounded_protocol(usize)` for protocols that are never sent, with a bounded receiver.
- `.add_readonly_unbounded_protocol()` for protocols that are never sent, with an unbounded receiver.

### fixed

- README compatibility table


## 0.8.9

### fixed

- Upped the `LatencyTracerPayload` buffer bound from 1 to 2, to avoid warnings when updates aren't in sync.

## 0.8.8

- First public release. Previous versions are untracked.
