# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_imsg_global_optspecs
    string join \n hub device= config= v/verbose q/quiet h/help V/version
end

function __fish_imsg_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_imsg_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_imsg_using_subcommand
    set -l cmd (__fish_imsg_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c imsg -n "__fish_imsg_needs_command" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_needs_command" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_needs_command" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_needs_command" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_needs_command" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_needs_command" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_needs_command" -s V -l version -d 'Print version'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "send" -d 'Send an SMS to a phone number'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "list" -d 'List messages in a folder'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "get" -d 'Fetch one message body by handle'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "delete" -d 'Delete (or undelete) a message by handle'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "contacts" -d 'Pull contacts from a phonebook'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "threads" -d 'Group inbox and sent messages into conversation threads'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "sync" -d 'Backfill the local store with all messages from the device since the last sync'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "unsync" -d 'Stop using the local store for reads; synced data is preserved by default'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "folders" -d 'List the MAP message folders on the device'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "hub" -d 'Start the iroh hub on this machine and print the node key for spokes'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "spoke" -d 'Manage spoke configuration for connecting to a remote hub'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "config" -d 'Inspect or modify local configuration'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "broker" -d 'Query the session broker'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "__broker_serve" -d 'Internal: session broker process, auto-started by the CLI — not for direct invocation'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "daemon" -d 'Manage the persistent background broker (opt-in; required for GUI use)'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "completions" -d 'Print a shell completion script to stdout'
complete -c imsg -n "__fish_imsg_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand send" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand send" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand send" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand send" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand send" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand send" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand list" -l from -d 'Filter by originating address' -r
complete -c imsg -n "__fish_imsg_using_subcommand list" -l since -d 'Filter to messages at or after this MAP timestamp (`YYYYMMDDTHHMMSS`)' -r
complete -c imsg -n "__fish_imsg_using_subcommand list" -l limit -d 'Maximum number of entries to return' -r
complete -c imsg -n "__fish_imsg_using_subcommand list" -l offset -d 'Skip the first N entries (pagination)' -r
complete -c imsg -n "__fish_imsg_using_subcommand list" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand list" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand list" -l unread -d 'Only unread messages'
complete -c imsg -n "__fish_imsg_using_subcommand list" -s l -l long -d 'Show the raw MAP handle for each message (for use with `get`/`delete`)'
complete -c imsg -n "__fish_imsg_using_subcommand list" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand list" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand list" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand list" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c imsg -n "__fish_imsg_using_subcommand get" -l folder -d 'Folder the handle lives in; defaults to inbox' -r -f -a "inbox\t'Received messages'
sent\t'Sent messages'
outbox\t'Pending outbound messages'
deleted\t'Deleted messages'"
complete -c imsg -n "__fish_imsg_using_subcommand get" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand get" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand get" -l mark-read -d 'Mark the message read after fetching'
complete -c imsg -n "__fish_imsg_using_subcommand get" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand get" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand get" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand get" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c imsg -n "__fish_imsg_using_subcommand delete" -l folder -d 'Folder the handle lives in; defaults to inbox' -r -f -a "inbox\t'Received messages'
sent\t'Sent messages'
outbox\t'Pending outbound messages'
deleted\t'Deleted messages'"
complete -c imsg -n "__fish_imsg_using_subcommand delete" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand delete" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand delete" -l undelete -d 'Restore a previously deleted message instead of deleting it'
complete -c imsg -n "__fish_imsg_using_subcommand delete" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand delete" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand delete" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand delete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l get -d 'Fetch a single contact: a PBAP handle live/via broker, or a cached UID once opted in (matches whatever `--list` just printed in that mode)' -r
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l lookup -d 'Reverse-lookup a contact by phone number' -r
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l path -d 'Phonebook to read. No effect on `--sync` (always the main phonebook)' -r -f -a "pb\t'Main phonebook'
ich\t'Incoming call history'
och\t'Outgoing call history'
mch\t'Missed call history'
cch\t'Combined call history'
spd\t'Speed-dial entries'
fav\t'Favourites'"
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l limit -d 'Maximum contacts per page; omit to show all. No effect on `--sync`' -r
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l page -d 'Page number (1-indexed). Ignored when `--limit` is not set. No effect on `--sync`' -r
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l list -d 'List handles/UIDs and names only, without full vCards'
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l sync -d 'Refresh the local contacts cache from the device. No effect from `--raw`/`--limit`/ `--page`; always syncs the main phonebook regardless of `--path`'
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l raw -d 'Show phone numbers as stored; skip E.164 normalisation. No effect on `--list`/`--sync`'
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand contacts" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c imsg -n "__fish_imsg_using_subcommand threads" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand threads" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand threads" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand threads" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand threads" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand threads" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand sync" -l folder -d 'Restrict the backfill to a single folder; omit to sync all folders' -r -f -a "inbox\t'Received messages'
sent\t'Sent messages'
outbox\t'Pending outbound messages'
deleted\t'Deleted messages'"
complete -c imsg -n "__fish_imsg_using_subcommand sync" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand sync" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand sync" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand sync" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand sync" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand sync" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c imsg -n "__fish_imsg_using_subcommand unsync" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand unsync" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand unsync" -l purge -d 'Delete the database file and all synced data in addition to disabling sync'
complete -c imsg -n "__fish_imsg_using_subcommand unsync" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand unsync" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand unsync" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand unsync" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand folders" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand folders" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand folders" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand folders" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand folders" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand folders" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand hub" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand hub" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand hub" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand hub" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand hub" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand hub" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -f -a "add" -d 'Persist the hub\'s iroh node key to the local config'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and not __fish_seen_subcommand_from add help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from add" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from add" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from add" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from add" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from add" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from help" -f -a "add" -d 'Persist the hub\'s iroh node key to the local config'
complete -c imsg -n "__fish_imsg_using_subcommand spoke; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -f -a "show" -d 'Print the resolved configuration'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -f -a "set-device" -d 'Persist the device MAC address to the user config file'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -f -a "setup" -d 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together'
complete -c imsg -n "__fish_imsg_using_subcommand config; and not __fish_seen_subcommand_from show set-device setup help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from show" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from show" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from show" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from show" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from show" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from set-device" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from set-device" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from set-device" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from set-device" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from set-device" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from set-device" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from setup" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from setup" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from setup" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from setup" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from setup" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from setup" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "show" -d 'Print the resolved configuration'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "set-device" -d 'Persist the device MAC address to the user config file'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "setup" -d 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together'
complete -c imsg -n "__fish_imsg_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -f -a "status" -d 'Report whether the broker is running and whether its MAP session is connected'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and not __fish_seen_subcommand_from status help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from status" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from status" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from status" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from help" -f -a "status" -d 'Report whether the broker is running and whether its MAP session is connected'
complete -c imsg -n "__fish_imsg_using_subcommand broker; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand __broker_serve" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand __broker_serve" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand __broker_serve" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand __broker_serve" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand __broker_serve" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand __broker_serve" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -f -a "start" -d 'Start the persistent broker. Detaches into the background by default; idempotent if already running'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -f -a "stop" -d 'Request a graceful stop. A no-op (not an error) if nothing is running'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -f -a "status" -d 'Report whether the daemon is running and whether its MAP session is connected'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -f -a "install" -d 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don\'t want OS-level supervision'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -f -a "uninstall" -d 'Unregister the daemon service. A no-op if it was never installed'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and not __fish_seen_subcommand_from start stop status install uninstall help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from start" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from start" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from start" -l foreground -d 'Stay attached instead of detaching — required under a process supervisor (e.g. a systemd unit). Stops on Ctrl-C, SIGTERM, or an IPC `Shutdown` request'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from start" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from start" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from start" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from stop" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from stop" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from stop" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from stop" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from stop" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from stop" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from status" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from status" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from status" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from status" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from status" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from status" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from install" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from install" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from install" -l system -d 'Register system-wide instead of for the current user only. Typically requires elevated privileges to install'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from install" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from install" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from install" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from install" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from uninstall" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from uninstall" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from uninstall" -l system -d 'Match the `--system`/user scope the service was installed with'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from uninstall" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from uninstall" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from uninstall" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from uninstall" -s h -l help -d 'Print help'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start the persistent broker. Detaches into the background by default; idempotent if already running'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from help" -f -a "stop" -d 'Request a graceful stop. A no-op (not an error) if nothing is running'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from help" -f -a "status" -d 'Report whether the daemon is running and whether its MAP session is connected'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from help" -f -a "install" -d 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don\'t want OS-level supervision'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from help" -f -a "uninstall" -d 'Unregister the daemon service. A no-op if it was never installed'
complete -c imsg -n "__fish_imsg_using_subcommand daemon; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand completions" -l device -d 'Override the configured device MAC address (RFCOMM only)' -r
complete -c imsg -n "__fish_imsg_using_subcommand completions" -l config -d 'Explicit config file path, overriding the layered default search' -r -F
complete -c imsg -n "__fish_imsg_using_subcommand completions" -l hub -d 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
complete -c imsg -n "__fish_imsg_using_subcommand completions" -s v -l verbose -d 'Increase logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand completions" -s q -l quiet -d 'Decrease logging verbosity'
complete -c imsg -n "__fish_imsg_using_subcommand completions" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "send" -d 'Send an SMS to a phone number'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "list" -d 'List messages in a folder'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "get" -d 'Fetch one message body by handle'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "delete" -d 'Delete (or undelete) a message by handle'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "contacts" -d 'Pull contacts from a phonebook'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "threads" -d 'Group inbox and sent messages into conversation threads'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "sync" -d 'Backfill the local store with all messages from the device since the last sync'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "unsync" -d 'Stop using the local store for reads; synced data is preserved by default'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "folders" -d 'List the MAP message folders on the device'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "hub" -d 'Start the iroh hub on this machine and print the node key for spokes'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "spoke" -d 'Manage spoke configuration for connecting to a remote hub'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "config" -d 'Inspect or modify local configuration'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "broker" -d 'Query the session broker'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "__broker_serve" -d 'Internal: session broker process, auto-started by the CLI — not for direct invocation'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "daemon" -d 'Manage the persistent background broker (opt-in; required for GUI use)'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "completions" -d 'Print a shell completion script to stdout'
complete -c imsg -n "__fish_imsg_using_subcommand help; and not __fish_seen_subcommand_from send list get delete contacts threads sync unsync folders hub spoke config broker __broker_serve daemon completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from spoke" -f -a "add" -d 'Persist the hub\'s iroh node key to the local config'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "show" -d 'Print the resolved configuration'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "set-device" -d 'Persist the device MAC address to the user config file'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "setup" -d 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from broker" -f -a "status" -d 'Report whether the broker is running and whether its MAP session is connected'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from daemon" -f -a "start" -d 'Start the persistent broker. Detaches into the background by default; idempotent if already running'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from daemon" -f -a "stop" -d 'Request a graceful stop. A no-op (not an error) if nothing is running'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from daemon" -f -a "status" -d 'Report whether the daemon is running and whether its MAP session is connected'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from daemon" -f -a "install" -d 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don\'t want OS-level supervision'
complete -c imsg -n "__fish_imsg_using_subcommand help; and __fish_seen_subcommand_from daemon" -f -a "uninstall" -d 'Unregister the daemon service. A no-op if it was never installed'
