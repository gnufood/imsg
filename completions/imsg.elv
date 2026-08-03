
use builtin;
use str;

set edit:completion:arg-completer[imsg] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'imsg'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'imsg'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand send 'Send an SMS to a phone number'
            cand list 'List messages in a folder'
            cand get 'Fetch one message body by handle'
            cand delete 'Delete (or undelete) a message by handle'
            cand contacts 'Pull contacts from a phonebook'
            cand threads 'Group inbox and sent messages into conversation threads'
            cand sync 'Backfill the local store with all messages from the device since the last sync'
            cand unsync 'Stop using the local store for reads; synced data is preserved by default'
            cand folders 'List the MAP message folders on the device'
            cand hub 'Start the iroh hub on this machine and print the node key for spokes'
            cand spoke 'Manage spoke configuration for connecting to a remote hub'
            cand config 'Inspect or modify local configuration'
            cand broker 'Query the session broker'
            cand __broker_serve 'Internal: session broker process, auto-started by the CLI — not for direct invocation'
            cand daemon 'Manage the persistent background broker (opt-in; required for GUI use)'
            cand completions 'Print a shell completion script to stdout'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;send'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;list'= {
            cand --from 'Filter by originating address'
            cand --since 'Filter to messages at or after this MAP timestamp (`YYYYMMDDTHHMMSS`)'
            cand --limit 'Maximum number of entries to return'
            cand --offset 'Skip the first N entries (pagination)'
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --unread 'Only unread messages'
            cand -l 'Show the raw MAP handle for each message (for use with `get`/`delete`)'
            cand --long 'Show the raw MAP handle for each message (for use with `get`/`delete`)'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'imsg;get'= {
            cand --folder 'Folder the handle lives in; defaults to inbox'
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --mark-read 'Mark the message read after fetching'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'imsg;delete'= {
            cand --folder 'Folder the handle lives in; defaults to inbox'
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --undelete 'Restore a previously deleted message instead of deleting it'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'imsg;contacts'= {
            cand --get 'Fetch a single contact: a PBAP handle live/via broker, or a cached UID once opted in (matches whatever `--list` just printed in that mode)'
            cand --lookup 'Reverse-lookup a contact by phone number'
            cand --path 'Phonebook to read. No effect on `--sync` (always the main phonebook)'
            cand --limit 'Maximum contacts per page; omit to show all. No effect on `--sync`'
            cand --page 'Page number (1-indexed). Ignored when `--limit` is not set. No effect on `--sync`'
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --list 'List handles/UIDs and names only, without full vCards'
            cand --sync 'Refresh the local contacts cache from the device. No effect from `--raw`/`--limit`/ `--page`; always syncs the main phonebook regardless of `--path`'
            cand --raw 'Show phone numbers as stored; skip E.164 normalisation. No effect on `--list`/`--sync`'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'imsg;threads'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;sync'= {
            cand --folder 'Restrict the backfill to a single folder; omit to sync all folders'
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'imsg;unsync'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --purge 'Delete the database file and all synced data in addition to disabling sync'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;folders'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;hub'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;spoke'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
            cand add 'Persist the hub''s iroh node key to the local config'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;spoke;add'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;spoke;help'= {
            cand add 'Persist the hub''s iroh node key to the local config'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;spoke;help;add'= {
        }
        &'imsg;spoke;help;help'= {
        }
        &'imsg;config'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
            cand show 'Print the resolved configuration'
            cand set-device 'Persist the device MAC address to the user config file'
            cand setup 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;config;show'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;config;set-device'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;config;setup'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;config;help'= {
            cand show 'Print the resolved configuration'
            cand set-device 'Persist the device MAC address to the user config file'
            cand setup 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;config;help;show'= {
        }
        &'imsg;config;help;set-device'= {
        }
        &'imsg;config;help;setup'= {
        }
        &'imsg;config;help;help'= {
        }
        &'imsg;broker'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
            cand status 'Report whether the broker is running and whether its MAP session is connected'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;broker;status'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;broker;help'= {
            cand status 'Report whether the broker is running and whether its MAP session is connected'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;broker;help;status'= {
        }
        &'imsg;broker;help;help'= {
        }
        &'imsg;__broker_serve'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;daemon'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
            cand start 'Start the persistent broker. Detaches into the background by default; idempotent if already running'
            cand stop 'Request a graceful stop. A no-op (not an error) if nothing is running'
            cand status 'Report whether the daemon is running and whether its MAP session is connected'
            cand install 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don''t want OS-level supervision'
            cand uninstall 'Unregister the daemon service. A no-op if it was never installed'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;daemon;start'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --foreground 'Stay attached instead of detaching — required under a process supervisor (e.g. a systemd unit). Stops on Ctrl-C, SIGTERM, or an IPC `Shutdown` request'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;daemon;stop'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;daemon;status'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;daemon;install'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --system 'Register system-wide instead of for the current user only. Typically requires elevated privileges to install'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;daemon;uninstall'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --system 'Match the `--system`/user scope the service was installed with'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'imsg;daemon;help'= {
            cand start 'Start the persistent broker. Detaches into the background by default; idempotent if already running'
            cand stop 'Request a graceful stop. A no-op (not an error) if nothing is running'
            cand status 'Report whether the daemon is running and whether its MAP session is connected'
            cand install 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don''t want OS-level supervision'
            cand uninstall 'Unregister the daemon service. A no-op if it was never installed'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;daemon;help;start'= {
        }
        &'imsg;daemon;help;stop'= {
        }
        &'imsg;daemon;help;status'= {
        }
        &'imsg;daemon;help;install'= {
        }
        &'imsg;daemon;help;uninstall'= {
        }
        &'imsg;daemon;help;help'= {
        }
        &'imsg;completions'= {
            cand --device 'Override the configured device MAC address (RFCOMM only)'
            cand --config 'Explicit config file path, overriding the layered default search'
            cand --hub 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`'
            cand -v 'Increase logging verbosity'
            cand --verbose 'Increase logging verbosity'
            cand -q 'Decrease logging verbosity'
            cand --quiet 'Decrease logging verbosity'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'imsg;help'= {
            cand send 'Send an SMS to a phone number'
            cand list 'List messages in a folder'
            cand get 'Fetch one message body by handle'
            cand delete 'Delete (or undelete) a message by handle'
            cand contacts 'Pull contacts from a phonebook'
            cand threads 'Group inbox and sent messages into conversation threads'
            cand sync 'Backfill the local store with all messages from the device since the last sync'
            cand unsync 'Stop using the local store for reads; synced data is preserved by default'
            cand folders 'List the MAP message folders on the device'
            cand hub 'Start the iroh hub on this machine and print the node key for spokes'
            cand spoke 'Manage spoke configuration for connecting to a remote hub'
            cand config 'Inspect or modify local configuration'
            cand broker 'Query the session broker'
            cand __broker_serve 'Internal: session broker process, auto-started by the CLI — not for direct invocation'
            cand daemon 'Manage the persistent background broker (opt-in; required for GUI use)'
            cand completions 'Print a shell completion script to stdout'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'imsg;help;send'= {
        }
        &'imsg;help;list'= {
        }
        &'imsg;help;get'= {
        }
        &'imsg;help;delete'= {
        }
        &'imsg;help;contacts'= {
        }
        &'imsg;help;threads'= {
        }
        &'imsg;help;sync'= {
        }
        &'imsg;help;unsync'= {
        }
        &'imsg;help;folders'= {
        }
        &'imsg;help;hub'= {
        }
        &'imsg;help;spoke'= {
            cand add 'Persist the hub''s iroh node key to the local config'
        }
        &'imsg;help;spoke;add'= {
        }
        &'imsg;help;config'= {
            cand show 'Print the resolved configuration'
            cand set-device 'Persist the device MAC address to the user config file'
            cand setup 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together'
        }
        &'imsg;help;config;show'= {
        }
        &'imsg;help;config;set-device'= {
        }
        &'imsg;help;config;setup'= {
        }
        &'imsg;help;broker'= {
            cand status 'Report whether the broker is running and whether its MAP session is connected'
        }
        &'imsg;help;broker;status'= {
        }
        &'imsg;help;__broker_serve'= {
        }
        &'imsg;help;daemon'= {
            cand start 'Start the persistent broker. Detaches into the background by default; idempotent if already running'
            cand stop 'Request a graceful stop. A no-op (not an error) if nothing is running'
            cand status 'Report whether the daemon is running and whether its MAP session is connected'
            cand install 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don''t want OS-level supervision'
            cand uninstall 'Unregister the daemon service. A no-op if it was never installed'
        }
        &'imsg;help;daemon;start'= {
        }
        &'imsg;help;daemon;stop'= {
        }
        &'imsg;help;daemon;status'= {
        }
        &'imsg;help;daemon;install'= {
        }
        &'imsg;help;daemon;uninstall'= {
        }
        &'imsg;help;completions'= {
        }
        &'imsg;help;help'= {
        }
    ]
    $completions[$command]
}
