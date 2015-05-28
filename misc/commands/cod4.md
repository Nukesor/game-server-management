
# Change gametype
In order to change the gametype you have to open the console first and type :
Code:
```
    /g_gametype
```

The aliases for different game types are:

koth = Headquarters
dm = Free for all
war = Team-deathmatch
sd = Search and destroy
sb = Sabotage
dom = Domination

And, now you want to change the game type from team `deathmatch` to `headquarters`, you have to type:
Code:
```
     /g_gametype koth
    /map_restart
        or
    /map mp_vacant
```

Remember, after that command, nothing will happen, and then you'll load any map or restart the map and you'll see that game type has been changed from team deathmatch to headquarters.
PS: /fast_restart won't work for changing gametypes.


# Change a map
Change ingame
`\map <mapname>`

Change in commandline:
`iw3mp.exe +set dedicated 2 +exec my_server.cfg +map mp_crash`


## Maps:

Version 1.5:
mp_convoy (Ambush)
mp_backlot (Backlot)
mp_bloc (Bloc)
mp_bog (Bog)
mp_countdown (Countdown)
mp_crash (Crash)
mp_crossfire (Crossfire)
mp_citystreets (District)
mp_farm (Downpour)
mp_overgrown (Overgrown)
mp_pipeline (Pipeline)
mp_shipment (Shipment)
mp_showdown (Showdown)
mp_strike (Strike)
mp_vacant (Vacant)
mp_cargoship (Wet Work)
mp_crash_snow (Winter Crash)

Version 1.6:
mp_broadcast (Broadcast)
mp_carentan (Chinatown)
mp_creek (Creek)
mp_killhouse (Killhouse)


# Server commands

`killserver`
`quit`
`map_restart`
`say "my message"`
`tell [clientnum] "my message to you"`

# Server settings

##map rotation
`sv_mapRotation gametype <gametype> map <mapname> gametype <gametype> map <mapname> ...`

`gametype` is one of:
"dm" - free for all deathmatch
"dom" - domination
"koth" - headquarters
"sab" - sabotage
"sd" - search & destroy
"war" - team deathmatch

sv_hostname "my server"
g_gametype <gametype>
sv_maxclients [1-32]
g_password "my password"
sv_voice [0-1]
scr_teambalance [0-1]
g_allowvote [0-1]
sv_punkbuster [0-1]
sv_minping [0-n] (milliseconds)
sv_maxping [0-n] (milliseconds)
sv_connectTimeout [0-n] (seconds)
sv_timeout [0-n] (seconds)

# Gameplay options

scr_oldschool [0-1]
scr_hardcore [0-1]
scr_game_spectatetype [0-2] (Disabled, Team/Players Only, Free)
scr_game_allowkillcam [0-1]
scr_team_fftype [0-3] (Disabled, Enabled, Reflect, Shared)

scr_game_perks [0-1]
scr_game_onlyheadshots [0-1]
scr_game_forceuav [0-1]
scr_game_hardpoints [0-1] (i.e. artillery, uav, helicopter)
scr_hardpoint_allowartillery [0-1]
scr_hardpoint_allowuav [0-1]
scr_hardpoint_allowhelicopter [0-1]

# Free for all deatmatch

scr_dm_scorelimit [0-n]
scr_dm_timelimit [0-n] (minutes)


# Domination

scr_dom_scorelimit [0-n]
scr_dom_timelimit [0-n] (minutes)

# Team deathmatch

scr_war_scorelimit [0-n]
scr_war_timelimit [0-n] (minutes)

# Sabotage

scr_sab_scorelimit [1-n] (points)
scr_sab_timelimit [0-n] (minutes)
scr_sab_roundswitch [0-n] (after how many rounds)
scr_sab_bombtimer [0-n] (seconds)
scr_sab_planttime [0-n] (seconds)
scr_sab_defusetime [0-n] (seconds)
scr_sab_hotpotato [0-1] (shared bomb timer)

# Search and destroy

scr_sd_scorelimit [1-n] (points)
scr_sd_timelimit [0-n] (minutes)
scr_sd_roundswitch [0-n] (number of rounds before switching teams)
scr_sd_bombtimer [0-n] (seconds)
scr_sd_planttime [0-n] (seconds)
scr_sd_defusetime [0-n] (seconds)
scr_sd_multibomb [0-1]

# Headquarters
scr_koth_scorelimit [1-n] (points)
scr_koth_timelimit [0-n] (minutes)
koth_autodestroytime [0-n] (seconds)
koth_kothmode [0-1] (classic mode, non-classic)
koth_spawntime [0-n] (seconds, hq spawn time)
