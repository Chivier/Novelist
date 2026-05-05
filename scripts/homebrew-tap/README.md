# homebrew-novelist

Homebrew tap for [Novelist](https://github.com/Saber-AI-Research/Novelist) — a
lightweight WYSIWYG Markdown desktop writing app for novelists.

## Install

```sh
brew tap saber-ai-research/novelist
brew install --cask novelist
```

Or in one line:

```sh
brew install --cask saber-ai-research/novelist/novelist
```

## Update

```sh
brew update
brew upgrade --cask novelist
```

The app also self-updates via the Tauri updater plugin, so `brew upgrade` is
optional once installed.

## Uninstall

```sh
brew uninstall --cask novelist
brew untap saber-ai-research/novelist
```

To wipe local state as well:

```sh
brew uninstall --cask --zap novelist
```

## Architectures

The cask publishes both Apple Silicon (`aarch64`) and Intel (`x64`) DMGs
produced by the [Novelist release pipeline][release].

The `Casks/novelist.rb` file in this repo is updated automatically by the
`bump-tap` job in [`.github/workflows/release.yml`][workflow] on every tagged
release. Manual edits will be overwritten on the next release.

[release]: https://github.com/Saber-AI-Research/Novelist/releases
[workflow]: https://github.com/Saber-AI-Research/Novelist/blob/main/.github/workflows/release.yml
