# Selfhosted Game Server

A collection of management scripts and configs for my game servers.

Each game has its own mini-binary for managing the startup, shutdown, backup or update of the respective server. \
All games are then launched in tmux sessions. That way you can attach to each server and still have an interactive tty.

### Misc

There is a `games.service` systemd service, which utilizes the `start.sh` and `stop.sh` mini-scripts. \
That way you can easily start/stop/backup servers on boot/shutdown

### Dependencies

Minecraft:
- jre8-openjdk-headless

ut2004:
- lib32-libstdc++5

CSGO:
- lib32-gcc-libs
- steamcmd


# Satisfactory

## Setup

Once the server is booted, you have to connect to it once via the actual Satisfactory interface.
You can then set a password and create a new world in the in-game interface.

If you want to load a local save on your server, do this:
1. Create a new world.
2. Make sure the world has been saved at least once, by checking the server's save directory.
3. Shut down the server
4. Remove the newly created savegame from your server's save folder.
5. Copy on of your local autosave files to the server's save folder.

## Saves and Config

The savegames of the server are located in `.config/Epic/FactoryGame/Saved/SaveGames/server`.

Server settings are saved in that directory as well in their own format.
Hence, it's not possible to have some immutable config for servers

# Factorio

To update Factorio, call `factorio update 1.1.37`.
