# Releasing

Releases are automated. Nobody edits a version number by hand.

## How a release happens

1. Every commit on `main` follows [Conventional Commits](https://www.conventionalcommits.org/): `feat:` adds a feature, `fix:` fixes a bug, `feat!:` or a `BREAKING CHANGE:` footer marks a breaking change.
2. On each push to `main`, the Release workflow runs release-please. It keeps one open pull request titled `chore(main): release X.Y.Z` that bumps the version in `package.json` and `src-tauri/Cargo.toml` and writes `CHANGELOG.md`.
3. Merging that pull request tags `vX.Y.Z` and creates the GitHub release. The same workflow then builds the Windows installer on `windows-latest`, signs the update bundle, and uploads the installer, its `.sig` signature and `latest.json` to the release.
4. Installed copies of Crowsnest read `latest.json` from the newest release, verify the signature against the public key compiled into the app, and offer the update.

While the version is below 1.0.0, `feat:` bumps the minor version and `fix:` bumps the patch version.

## One-time repository setup

These secrets must exist in the `release` environment under Settings, Environments, with deployments limited to the `main` branch. Only the publish job runs in that environment, so pull request builds never see them:

| Secret | What it holds |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of the private update-signing key (`~/.tauri/crowsnest.key` on the maintainer's machine). |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | The key's password (`~/.tauri/crowsnest.key.password` on the maintainer's machine until it moves to a password manager). |
| `WG_APPLICATION_ID` | The Wargaming developer application ID compiled into release builds as the default. |

The public half of the signing key lives in `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`. Losing the private key means installed copies can no longer verify updates, so keep a backup of the key and its password outside the repository, in separate places. Rotating it requires shipping one release signed with the old key that carries the new public key.

## Building locally

```powershell
npm ci
npm run tauri build
```

The installer lands in `src-tauri/target/release/bundle/nsis/`. Because `createUpdaterArtifacts` is on, bundling needs `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` in the environment. Without the key, `npm run tauri build -- --no-bundle` still produces the plain executable.
