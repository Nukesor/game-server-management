# Selfhosted Game Server

A collection of management scripts and configs for my game servers.

Each game has its own mini-binary for managing the startup, shutdown, backup or update of the respective server. \
All games are then launched in tmux sessions. That way you can attach to each server and still have an interactive tty.

## Updates

To update Factorio, call `factorio update 1.1.37`.

## Misc

There is a `games.service` systemd service, which utilizes the `start.sh` and `stop.sh` mini-scripts. \
That way you can easily start/stop/backup servers on boot/shutdown

## Dependencies

Some games require a few dependencies. These are listed in the `misc/dependencies.txt` folder.
These dependencies are currently for Arch Linux only.
