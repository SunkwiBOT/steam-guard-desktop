# Security policy

## Supported versions

Only the latest release gets security fixes. The app offers new versions in **Settings
› Updates**; please check the problem still happens on the latest build before reporting.

## Reporting a vulnerability

Please do not report a vulnerability in a public issue, discussion or pull request.

Report it privately with GitHub's private vulnerability reporting: open the repository's
**Security** tab and click **Report a vulnerability**. Only you and the maintainers see
the report, and it stays private until its advisory is published.

## What to include

- The version (**Settings › Updates**, or the first line of the logs) and the operating
  system.
- The part concerned, for example the vault password and lock, the linking/finalizing of
  an authenticator, imports/backups, or the updater.
- The steps to reproduce it, or a proof of concept, and what an attacker gains.
- Logs when they help.

Never send real `shared_secret`, `identity_secret`, revocation codes, refresh tokens,
the vault password, a maFile, or the database file itself.

## After a report

The report is discussed in its private advisory. Once a fix is released, the advisory
is published, with credit to you if you wish.

## Scope

This repository's code: the desktop app, its vault/backup/import formats, and the
release and update process. Problems in Steam itself belong to Valve.
