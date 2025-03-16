# nixpkgs-prs Bot

## Overview

`nixpkgs-prs` is a bot that fetches pull request information from the
[nixpkgs](https://github.com/NixOS/nixpkgs) repository. It supports optional
posting to [Bluesky (bsky)](https://bsky.app) and [Fediverse (fedi)](https://fediverse.to) platforms, which are gated behind feature flags.

## Features

- **Fetch PR Data**: Retrieves pull request information from the nixpkgs repository.
- **Post to Bluesky** (*Requires `post-bsky` feature*): Publishes updates to Bluesky.
- **Post to Fediverse** (*Requires `post-fedi` feature*): Publishes updates to Fediverse.

## Configuration

If you are running in a container or systemd service you may consider setting
the following environment variables, otherwise they are accessible as flags:

- **Bluesky (`post-bsky` feature)**:
  - `BSKY_EMAIL`: Your Bluesky email.
  - `BSKY_PASSWORD`: Your Bluesky app password.
- **Fediverse (`post-fedi` feature)**:
  - `FEDI_INSTANCE`: The URL of your Fediverse instance.
  - `FEDI_TOKEN`: Your access token.

## License

This project is licensed under the EUPL-1.2 License.

## Contributing

Contributions are welcome! Feel free to open issues or submit PRs.
