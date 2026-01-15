# Git Utilities

## util/alias

Add git aliases globally.

```bash
./util/alias <name> <command>
```

Example:
```bash
./util/alias co checkout
./util/alias st status
```

## util/git-config-d

Ensure XDG git config includes `git/config.d/*.conf` files.

Creates:
- `~/.config/git/config` (if it doesn't exist)
- `~/.config/git/config.d/` directory
- Adds `[include] path = ~/.config/git/config.d/*.conf` to the config

Usage:
```bash
./util/git-config-d
```

This allows you to drop `.conf` files into `~/.config/git/config.d/` and they will be automatically included in your git configuration.
