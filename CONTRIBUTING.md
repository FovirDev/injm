# Contributing Guidance

All contributions to `injm` are welcomed. Here are notes of contribution.

## Make Contribution

0. Fork this repository
1. Clone and enter the directory

   ```bash
   git clone git@github.com:<your-username>/injm.git && cd injm/
   ```

   > Replace `<your-username>` with your GitHub username.

2. Add and update upstream remote

   ```bash
   git remote add upstream https://github.com/FovirDev/injm.git
   git fetch upstream
   ```

3. Create a new branch

   ```bash
   git switch --create <branch-name> upstream/develop
   ```

   > `<branch-name>` should follow [Branch Policy](#branch-policy).

4. Development environment setup can be found in [Development Environment](#development-environment) section
5. Commit (follows [Conventional Commits](https://www.conventionalcommits.org/)) and push changes to your forked repository
6. Create a pull request targeted to `develop` branch
7. Wait for review and approval

## Development Environment

### Setup

Quick start with `direnv` and `flake`:

```bash
direnv allow
```

Alternatively, please install `cargo` manually.

`injm` also contains `pre-commit` to perform lint before pushing. So, it is recommended to set up `pre-commit` using

```bash
pre-commit install -f
pre-commit install --hook-type pre-push -f
```

### `justfile`

A `justfile` can be found in the root directory of `injm`, where contains useful commands during development that can be executed with `just`.

- Run the program

  ```bash
  just run help
  ```

- Build binary

  ```bash
  just build # Debug
  just build-release # Release
  ```

- Testing

  ```bash
  just test
  ```

- Lint

  ```bash
  just lint
  ```

- Fix

  ```bash
  just fix
  ```

- Generate Markdown table of contents

  ```bash
  just toc
  ```

- Release a new version (See [Release Policy](#release-policy))

  ```bash
  just release X.Y.Z
  ```

## Release Policy

Tags of `injm` follow `vMAJOR.MINOR.PATCH` format.

To create a new release, please run the following commands in `main` branch.

```bash
just release <new-version>
cargo publish
```

> `<new-version>` does not contain `v` prefix. For example, to release `v1.0.0`, use `just release 1.0.0` instead of `just release v1.0.0`

## Branch Policy

`injm` uses `main` branch for stable release version to support `nix profile install`, and development changes should go to `develop` branch.

### Branch Naming Rule

- `feature/*`: For new added features (e.g. `feature/inject-config`)
- `fix/*`: For bug fixes (e.g. `fix/comment-extraction`)
- `docs/*`: For document-related changes (e.g. `docs/fix-typo-in-readme`)
- `chore/*`: For misc changes (e.g. `chore/update-flake`)
