# Releases and versioning

## Version format

Manual and release workflow versions use `YY.N`:

- `YY` is the two-digit year in UTC.
- `N` is the release sequence for that year.

For example, a release in 2026 can use `26.1`, followed by `26.2`. The desktop package version is represented as a semver-compatible value such as `26.1.0` where the packaging tool requires three components.

The workflow derives the version for manually dispatched release work from a tag and uses a short SHA for ordinary push and pull-request validation. Check the workflow and generated metadata when a release identifier matters.

## Release matrix

Keep these synchronized:

- application version and generated update metadata;
- Windows x64 and ARM64 artifacts;
- Linux x64 and ARM64 artifacts;
- macOS x64 and ARM64 artifacts;
- exact asset names and architecture labels;
- validators, checksums, provenance, and release notes.

Portable archives and installer/update targets are not interchangeable. Confirm which artifacts are intended for installation, portable use, or update distribution.

## Signing

Signing is optional in the default build path and depends on the relevant repository secrets and platform credentials. An unsigned artifact must be described as unsigned; it is not equivalent to a signed production release.

## Before publishing

Run the local gates, inspect the complete CI matrix, verify the published asset set, and test the artifact on each supported architecture that the release claims to support. Do not call a release complete from a single local build.
