
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'imsg' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'imsg'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'imsg' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('send', 'send', [CompletionResultType]::ParameterValue, 'Send an SMS to a phone number')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List messages in a folder')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Fetch one message body by handle')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete (or undelete) a message by handle')
            [CompletionResult]::new('contacts', 'contacts', [CompletionResultType]::ParameterValue, 'Pull contacts from a phonebook')
            [CompletionResult]::new('threads', 'threads', [CompletionResultType]::ParameterValue, 'Group inbox and sent messages into conversation threads')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Backfill the local store with all messages from the device since the last sync')
            [CompletionResult]::new('unsync', 'unsync', [CompletionResultType]::ParameterValue, 'Stop using the local store for reads; synced data is preserved by default')
            [CompletionResult]::new('folders', 'folders', [CompletionResultType]::ParameterValue, 'List the MAP message folders on the device')
            [CompletionResult]::new('hub', 'hub', [CompletionResultType]::ParameterValue, 'Start the iroh hub on this machine and print the node key for spokes')
            [CompletionResult]::new('spoke', 'spoke', [CompletionResultType]::ParameterValue, 'Manage spoke configuration for connecting to a remote hub')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Inspect or modify local configuration')
            [CompletionResult]::new('broker', 'broker', [CompletionResultType]::ParameterValue, 'Query the session broker')
            [CompletionResult]::new('__broker_serve', '__broker_serve', [CompletionResultType]::ParameterValue, 'Internal: session broker process, auto-started by the CLI — not for direct invocation')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Manage the persistent background broker (opt-in; required for GUI use)')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Print a shell completion script to stdout')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;send' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;list' {
            [CompletionResult]::new('--from', '--from', [CompletionResultType]::ParameterName, 'Filter by originating address')
            [CompletionResult]::new('--since', '--since', [CompletionResultType]::ParameterName, 'Filter to messages at or after this MAP timestamp (`YYYYMMDDTHHMMSS`)')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Maximum number of entries to return')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'Skip the first N entries (pagination)')
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--unread', '--unread', [CompletionResultType]::ParameterName, 'Only unread messages')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Show the raw MAP handle for each message (for use with `get`/`delete`)')
            [CompletionResult]::new('--long', '--long', [CompletionResultType]::ParameterName, 'Show the raw MAP handle for each message (for use with `get`/`delete`)')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'imsg;get' {
            [CompletionResult]::new('--folder', '--folder', [CompletionResultType]::ParameterName, 'Folder the handle lives in; defaults to inbox')
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--mark-read', '--mark-read', [CompletionResultType]::ParameterName, 'Mark the message read after fetching')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'imsg;delete' {
            [CompletionResult]::new('--folder', '--folder', [CompletionResultType]::ParameterName, 'Folder the handle lives in; defaults to inbox')
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--undelete', '--undelete', [CompletionResultType]::ParameterName, 'Restore a previously deleted message instead of deleting it')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'imsg;contacts' {
            [CompletionResult]::new('--get', '--get', [CompletionResultType]::ParameterName, 'Fetch a single contact: a PBAP handle live/via broker, or a cached UID once opted in (matches whatever `--list` just printed in that mode)')
            [CompletionResult]::new('--lookup', '--lookup', [CompletionResultType]::ParameterName, 'Reverse-lookup a contact by phone number')
            [CompletionResult]::new('--path', '--path', [CompletionResultType]::ParameterName, 'Phonebook to read. No effect on `--sync` (always the main phonebook)')
            [CompletionResult]::new('--limit', '--limit', [CompletionResultType]::ParameterName, 'Maximum contacts per page; omit to show all. No effect on `--sync`')
            [CompletionResult]::new('--page', '--page', [CompletionResultType]::ParameterName, 'Page number (1-indexed). Ignored when `--limit` is not set. No effect on `--sync`')
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--list', '--list', [CompletionResultType]::ParameterName, 'List handles/UIDs and names only, without full vCards')
            [CompletionResult]::new('--sync', '--sync', [CompletionResultType]::ParameterName, 'Refresh the local contacts cache from the device. No effect from `--raw`/`--limit`/ `--page`; always syncs the main phonebook regardless of `--path`')
            [CompletionResult]::new('--raw', '--raw', [CompletionResultType]::ParameterName, 'Show phone numbers as stored; skip E.164 normalisation. No effect on `--list`/`--sync`')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'imsg;threads' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;sync' {
            [CompletionResult]::new('--folder', '--folder', [CompletionResultType]::ParameterName, 'Restrict the backfill to a single folder; omit to sync all folders')
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'imsg;unsync' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--purge', '--purge', [CompletionResultType]::ParameterName, 'Delete the database file and all synced data in addition to disabling sync')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;folders' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;hub' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;spoke' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Persist the hub''s iroh node key to the local config')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;spoke;add' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;spoke;help' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Persist the hub''s iroh node key to the local config')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;spoke;help;add' {
            break
        }
        'imsg;spoke;help;help' {
            break
        }
        'imsg;config' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Print the resolved configuration')
            [CompletionResult]::new('set-device', 'set-device', [CompletionResultType]::ParameterValue, 'Persist the device MAC address to the user config file')
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;config;show' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;config;set-device' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;config;setup' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;config;help' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Print the resolved configuration')
            [CompletionResult]::new('set-device', 'set-device', [CompletionResultType]::ParameterValue, 'Persist the device MAC address to the user config file')
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;config;help;show' {
            break
        }
        'imsg;config;help;set-device' {
            break
        }
        'imsg;config;help;setup' {
            break
        }
        'imsg;config;help;help' {
            break
        }
        'imsg;broker' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Report whether the broker is running and whether its MAP session is connected')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;broker;status' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;broker;help' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Report whether the broker is running and whether its MAP session is connected')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;broker;help;status' {
            break
        }
        'imsg;broker;help;help' {
            break
        }
        'imsg;__broker_serve' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;daemon' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the persistent broker. Detaches into the background by default; idempotent if already running')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Request a graceful stop. A no-op (not an error) if nothing is running')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Report whether the daemon is running and whether its MAP session is connected')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don''t want OS-level supervision')
            [CompletionResult]::new('uninstall', 'uninstall', [CompletionResultType]::ParameterValue, 'Unregister the daemon service. A no-op if it was never installed')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;daemon;start' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--foreground', '--foreground', [CompletionResultType]::ParameterName, 'Stay attached instead of detaching — required under a process supervisor (e.g. a systemd unit). Stops on Ctrl-C, SIGTERM, or an IPC `Shutdown` request')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;daemon;stop' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;daemon;status' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;daemon;install' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--system', '--system', [CompletionResultType]::ParameterName, 'Register system-wide instead of for the current user only. Typically requires elevated privileges to install')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;daemon;uninstall' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--system', '--system', [CompletionResultType]::ParameterName, 'Match the `--system`/user scope the service was installed with')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'imsg;daemon;help' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the persistent broker. Detaches into the background by default; idempotent if already running')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Request a graceful stop. A no-op (not an error) if nothing is running')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Report whether the daemon is running and whether its MAP session is connected')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don''t want OS-level supervision')
            [CompletionResult]::new('uninstall', 'uninstall', [CompletionResultType]::ParameterValue, 'Unregister the daemon service. A no-op if it was never installed')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;daemon;help;start' {
            break
        }
        'imsg;daemon;help;stop' {
            break
        }
        'imsg;daemon;help;status' {
            break
        }
        'imsg;daemon;help;install' {
            break
        }
        'imsg;daemon;help;uninstall' {
            break
        }
        'imsg;daemon;help;help' {
            break
        }
        'imsg;completions' {
            [CompletionResult]::new('--device', '--device', [CompletionResultType]::ParameterName, 'Override the configured device MAC address (RFCOMM only)')
            [CompletionResult]::new('--config', '--config', [CompletionResultType]::ParameterName, 'Explicit config file path, overriding the layered default search')
            [CompletionResult]::new('--hub', '--hub', [CompletionResultType]::ParameterName, 'Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Increase logging verbosity')
            [CompletionResult]::new('-q', '-q', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('--quiet', '--quiet', [CompletionResultType]::ParameterName, 'Decrease logging verbosity')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'imsg;help' {
            [CompletionResult]::new('send', 'send', [CompletionResultType]::ParameterValue, 'Send an SMS to a phone number')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List messages in a folder')
            [CompletionResult]::new('get', 'get', [CompletionResultType]::ParameterValue, 'Fetch one message body by handle')
            [CompletionResult]::new('delete', 'delete', [CompletionResultType]::ParameterValue, 'Delete (or undelete) a message by handle')
            [CompletionResult]::new('contacts', 'contacts', [CompletionResultType]::ParameterValue, 'Pull contacts from a phonebook')
            [CompletionResult]::new('threads', 'threads', [CompletionResultType]::ParameterValue, 'Group inbox and sent messages into conversation threads')
            [CompletionResult]::new('sync', 'sync', [CompletionResultType]::ParameterValue, 'Backfill the local store with all messages from the device since the last sync')
            [CompletionResult]::new('unsync', 'unsync', [CompletionResultType]::ParameterValue, 'Stop using the local store for reads; synced data is preserved by default')
            [CompletionResult]::new('folders', 'folders', [CompletionResultType]::ParameterValue, 'List the MAP message folders on the device')
            [CompletionResult]::new('hub', 'hub', [CompletionResultType]::ParameterValue, 'Start the iroh hub on this machine and print the node key for spokes')
            [CompletionResult]::new('spoke', 'spoke', [CompletionResultType]::ParameterValue, 'Manage spoke configuration for connecting to a remote hub')
            [CompletionResult]::new('config', 'config', [CompletionResultType]::ParameterValue, 'Inspect or modify local configuration')
            [CompletionResult]::new('broker', 'broker', [CompletionResultType]::ParameterValue, 'Query the session broker')
            [CompletionResult]::new('__broker_serve', '__broker_serve', [CompletionResultType]::ParameterValue, 'Internal: session broker process, auto-started by the CLI — not for direct invocation')
            [CompletionResult]::new('daemon', 'daemon', [CompletionResultType]::ParameterValue, 'Manage the persistent background broker (opt-in; required for GUI use)')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Print a shell completion script to stdout')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'imsg;help;send' {
            break
        }
        'imsg;help;list' {
            break
        }
        'imsg;help;get' {
            break
        }
        'imsg;help;delete' {
            break
        }
        'imsg;help;contacts' {
            break
        }
        'imsg;help;threads' {
            break
        }
        'imsg;help;sync' {
            break
        }
        'imsg;help;unsync' {
            break
        }
        'imsg;help;folders' {
            break
        }
        'imsg;help;hub' {
            break
        }
        'imsg;help;spoke' {
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Persist the hub''s iroh node key to the local config')
            break
        }
        'imsg;help;spoke;add' {
            break
        }
        'imsg;help;config' {
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Print the resolved configuration')
            [CompletionResult]::new('set-device', 'set-device', [CompletionResultType]::ParameterValue, 'Persist the device MAC address to the user config file')
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Interactively pick a paired device, resolve its MAP/PBAP channels over SDP, and persist address + both channels together')
            break
        }
        'imsg;help;config;show' {
            break
        }
        'imsg;help;config;set-device' {
            break
        }
        'imsg;help;config;setup' {
            break
        }
        'imsg;help;broker' {
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Report whether the broker is running and whether its MAP session is connected')
            break
        }
        'imsg;help;broker;status' {
            break
        }
        'imsg;help;__broker_serve' {
            break
        }
        'imsg;help;daemon' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start the persistent broker. Detaches into the background by default; idempotent if already running')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Request a graceful stop. A no-op (not an error) if nothing is running')
            [CompletionResult]::new('status', 'status', [CompletionResultType]::ParameterValue, 'Report whether the daemon is running and whether its MAP session is connected')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Register the daemon with the native OS service manager (systemd/launchd/OpenRC/ rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg daemon start`/`stop` alone fully serve users who don''t want OS-level supervision')
            [CompletionResult]::new('uninstall', 'uninstall', [CompletionResultType]::ParameterValue, 'Unregister the daemon service. A no-op if it was never installed')
            break
        }
        'imsg;help;daemon;start' {
            break
        }
        'imsg;help;daemon;stop' {
            break
        }
        'imsg;help;daemon;status' {
            break
        }
        'imsg;help;daemon;install' {
            break
        }
        'imsg;help;daemon;uninstall' {
            break
        }
        'imsg;help;completions' {
            break
        }
        'imsg;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
