# Releasing

Releases are automated. Nobody edits a version number by hand.

## How a release happens

1. Every commit on `main` follows [Conventional Commits](https://www.conventionalcommits.org/): `feat:` adds a feature, `fix:` fixes a bug, `feat!:` or a `BREAKING CHANGE:` footer marks a breaking change.
2. On each push to `main`, the Release workflow runs release-please. It keeps one open pull request titled `chore(main): release X.Y.Z` that bumps the version in `package.json` and `src-tauri/Cargo.toml` and writes `CHANGELOG.md`.
3. Merging that pull request tags `vX.Y.Z` and creates the GitHub release. Two jobs then finish it:
   - **Build installer** runs on `windows-latest` and compiles the unsigned NSIS installer. It is the only job that runs project dependencies, and it never sees the signing key.
   - **Sign and publish** runs on `ubuntu-latest` and runs no project code beyond the Tauri CLI, installed from the lockfile with install scripts off. It signs the installer with the version bound into the signature, writes `latest.json`, and uploads the installer, its `.sig` and `latest.json` to the release.
4. Installed copies of Crowsnest read `latest.json` from the newest release. They verify the signature against the public key compiled into the app, and refuse an update whose signed version differs from the version `latest.json` announces (`requireSignedVersion`).

While the version is below 1.0.0, `feat:` bumps the minor version and `fix:` bumps the patch version.

## One-time repository setup

Two environments under Settings, Environments, each with deployments limited to the `main` branch, hold the secrets. Pull request runs reach neither, and each release job reaches only its own:

| Environment | Secret | What it holds |
|---|---|---|
| `build` | `WG_APPLICATION_ID` | The Wargaming developer application ID compiled into release builds as the default. |
| `release` | `TAURI_SIGNING_PRIVATE_KEY` | Contents of the private update-signing key (`~/.tauri/crowsnest.key` on the maintainer's machine). |
| `release` | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | The key's password (`~/.tauri/crowsnest.key.password` on the maintainer's machine until it moves to a password manager). |

Under Settings, Actions, General, workflow permissions stay read-only, with "Allow GitHub Actions to create and approve pull requests" on so release-please can open its pull request.

The public half of the signing key lives in `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`. Losing the private key means installed copies can no longer verify updates, so keep a backup of the key and its password outside the repository, in separate places. Rotating it requires shipping one release signed with the old key that carries the new public key.

## Building locally

```powershell
npm ci
npm run tauri build
```

The installer lands in `src-tauri/target/release/bundle/nsis/` and needs no key. To produce an update signature by hand, as the release job does:

```powershell
npx tauri signer sign --app-version 0.1.0 -f $HOME\.tauri\crowsnest.key src-tauri\target\release\bundle\nsis\Crowsnest_0.1.0_x64-setup.exe
eleaseundle
sis\Crowsnest_0.1.0_x64-setup.exe
```

The CLI asks for the key's password, or reads it from `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
