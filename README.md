<p align="center"><img src="docs/logo/haunt-mark.svg" width="72" alt=""></p>

<h1 align="center">Haunt</h1>

<p align="center">A git client that also maps your code.</p>

Haunt is a desktop git client for Windows, macOS and Linux. Next to the usual work (staging, committing, branches,
merging, rebasing, pull requests) it reads your project by function: where each function is written, what it calls,
what calls it, and how the project's parts import each other. So a change is not just a list of files. Haunt shows
which functions it touches, what could break, which tests reach it and who knows that part of the code.

Everything runs on your machine. Haunt sends nothing anywhere unless you ask it to, and it runs none of a project's own
code until you trust its folder.

## What it does

- **Git, all of it.** A commit list with its graph, local changes with line staging, branches, stashes, tags, fetch,
  pull and push, cherry-pick, revert, reset, merge, interactive rebase, a conflict view, worktrees, submodules, bisect
  and patches. Undo for discards, dropped stashes and resets.
- **Changes by function.** Every commit, range or set of local changes, read function by function: what was added,
  changed, moved or removed, what calls it, which tests reach it, and a risk line that says where to look first.
- **Maps of the code.** Layers, a zoomable canvas of boxes in boxes, call paths from the entry points, the imports
  between parts with their rule breaks and cycles, unused code, and the HTTP routes with their handlers.
- **History that follows the code.** A function's history across renames and moves, where it came from, blame that
  steps back, who knows each part, and release notes between two tags.
- **Reviews without switching branches.** Pull requests fetched as refs and read with every lens, a branch reviewed
  function by function, and suggested reviewers.
- **Made for your own setup.** Tabs and profiles, themes from Nuxt UI, your editor one click away, and a command
  palette for everything.

All of it is described in [docs/features.md](docs/features.md); what comes next is in [docs/roadmap.md](docs/roadmap.md).

## Languages

TypeScript and JavaScript (with the script blocks of Vue, Svelte and Astro files) are read with the TypeScript
compiler, so their calls are exact. PHP, Rust, Lua, Python, Go, Java, C#, Ruby, C and C++ are read with tree-sitter,
and their calls are matched by name. No configuration is needed; a `haunt.config.json` can add your own layers and import rules.

## Install

Download the installer for your system from the [releases](https://github.com/razerghost/haunt/releases): `.msi` or
`.exe` for Windows, `.dmg` for macOS, `.deb`, `.rpm` or `.AppImage` for Linux. The builds are not signed yet, so
Windows and macOS warn the first time you open Haunt.

On Arch and its derivatives (CachyOS, EndeavourOS, Manjaro), each release has a package for pacman
(`haunt-bin-<version>-1-x86_64.pkg.tar.zst`, under the release's assets). It is not signed, and pacman installs an
unsigned package only from a file it has, so download it first:

```bash
curl -LO https://github.com/RazerGhost/haunt/releases/download/v0.3.2/haunt-bin-0.3.2-1-x86_64.pkg.tar.zst
sudo pacman -U haunt-bin-0.3.2-1-x86_64.pkg.tar.zst
```

Or build the same package from this repository with `makepkg -si` in `packaging/aur/haunt-bin`. It is meant for the
AUR as `haunt-bin` (`paru -S haunt-bin`), which is not up yet.

Haunt uses the git on your machine (2.24 or later) and reads TypeScript with the Node you already have (22.18 or
later). Without Node it uses Bun or Deno; running a project's tests still needs Node. With none of the three, it still
reads PHP, Rust, Lua, Python, Go, Java, C#, Ruby, C and C++. It checks these when it starts and says what is missing.

Open a repository with the + next to the tabs, or run `haunt <folder>`. Install the project's dependencies first, so
the imports between its packages resolve.

## Privacy and trust

- Haunt talks to no service on its own. What goes online is what you switch on or pick: pull request status through
  your own `gh`, `glab` or `tea`, a daily check for a new Haunt release, a theme's fonts from coolLabs Fonts, and an
  icon set other than Lucide, downloaded once from the Iconify API. Fetch, pull and push go to your remotes, as git
  does.
- Your passwords stay with git. Haunt uses your credential manager, keychain and SSH agent, and when git asks
  something, the question shows in Haunt and the answer goes straight back to git.
- Until you trust a folder, Haunt runs none of its code: no hooks, filters or scripts, and git runs with everything
  the repository's own config could make it run switched off. Before you trust a folder, Haunt shows what it would
  run.

## Build it yourself

```bash
pnpm install
pnpm app
```

`pnpm app` starts Haunt with hot reload; `pnpm app:build` builds an installer. You need Node 22.18 or later, pnpm and
Rust, and on Windows the Visual Studio C++ Build Tools. How Haunt is put together, and the checks to run before a
change, are in [docs/development.md](docs/development.md).

## License

Apache License 2.0, see [LICENSE](LICENSE).
