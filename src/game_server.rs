use crate::{config::Config, errors::*};

/// Common trait for game server lifecycle management.
///
/// This trait standardizes the common operations that all game server binaries perform:
/// startup, shutdown, backup, and optional update functionality.
///
/// Each game implementation handles:
/// - Session management via tmux
/// - Game-specific configuration deployment
/// - Server process startup/shutdown
/// - Backup creation when applicable
pub trait GameServer {
    /// Get the configuration for this game server.
    fn config(&self) -> &Config;

    /// Passthrough to Config::session_name
    fn session_name(&self) -> String {
        self.config().session_name()
    }

    /// Start the game server.
    ///
    /// Wrapper around startup_inner with logging and other stuff.
    fn startup(&self) -> Result<()> {
        info!("{} - Starting up server", self.config().session_name());
        self.startup_inner()?;
        info!("{} - Server has started", self.config().session_name());

        Ok(())
    }

    /// This should:
    /// 1. Check that no session is already running
    /// 2. Deploy any necessary configuration files
    /// 3. Create a tmux session
    /// 4. Start the server process
    fn startup_inner(&self) -> Result<()>;

    /// Backup the game server.
    ///
    /// Wrapper around backup_inner with logging and other stuff.
    fn backup(&self) -> Result<()> {
        info!("{} - Backing up server", self.config().session_name());
        self.backup_inner()?;
        info!("{} - Backup has been created", self.config().session_name());

        Ok(())
    }

    /// Create a backup of the game server data.
    fn backup_inner(&self) -> Result<()> {
        bail!(
            "{} - Backup functionality is not implemented",
            self.config().session_name()
        );
    }

    /// Backup the game server.
    ///
    /// Wrapper around backup_inner with logging and other stuff.
    fn update(&self) -> Result<()> {
        info!("{} - Updating server", self.config().session_name());
        self.update_inner()?;
        info!("{} - Server has been updated", self.config().session_name());

        Ok(())
    }

    /// Update the game server.
    fn update_inner(&self) -> Result<()> {
        bail!(
            "{} - Update functionality is not implemented",
            self.config().session_name()
        );
    }

    /// Shutting down the game server.
    ///
    /// Wrapper around shutdown_inner with logging and other stuff.
    fn shutdown(&self) -> Result<()> {
        info!("{} - Shutting down server", self.config().session_name());
        self.shutdown_inner()?;
        info!(
            "{} - Server has been shut down",
            self.config().session_name()
        );

        Ok(())
    }

    /// Shutdown the game server gracefully.
    ///
    /// The default implementation uses Ctrl-C before exiting the tmux session. This works for most
    /// games, but quite a few games that need custom shutdown logic (like Terraria using the "exit"
    /// command).
    fn shutdown_inner(&self) -> Result<()>;
}
