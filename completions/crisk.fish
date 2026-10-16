# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_crisk_global_optspecs
    string join \n q/quiet h/help V/version
end

function __fish_crisk_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_crisk_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_crisk_using_subcommand
    set -l cmd (__fish_crisk_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c crisk -n "__fish_crisk_needs_command" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c crisk -n "__fish_crisk_needs_command" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "collect" -d 'Collect the facts of one change: paths from git, map and policy from the base revision, declarations and label events from the forge, labels from signal files'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "evaluate" -d 'Evaluate a facts document into an assessment, offline'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "assess" -d 'Collect and evaluate one change'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "render" -d 'Render an assessment (or a Statement) in another format, after validating it'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "validate" -d 'Validate a facts document, assessment, Statement, map, policy, requirements document or emergency population'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "compile" -d 'Compile the tier policy into acc\'s tier section, a requirements document or prose'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "population" -d 'List the emergency population: merged changes carrying the emergency label in a window'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "completions" -d 'Generate shell completions'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "manpage" -d 'Generate man pages'
complete -c crisk -n "__fish_crisk_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l github -d 'Read the change from this GitHub repository (OWNER/REPO); requires --pr' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l pr -d 'The pull request number' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l gitlab -d 'Read the change from this GitLab project (GROUP/PROJECT); requires --mr' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l mr -d 'The merge request IID' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l gitlab-url -d 'The GitLab API base URL (https only)' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l max-pages -d 'Maximum pages per list endpoint (1..=1000)' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l base -d 'The target branch commit the map and policy are read from (default: the forge change\'s base)' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l head -d 'The change\'s head (default: the forge change\'s head, else HEAD)' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l checkout -d 'The local clone' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l declarations -d 'Declarations from a YAML file instead of the change\'s description' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand collect" -l signal -d 'A signal file: sarif=PATH, privacy-datamap=PATH or acc=PATH (repeatable)' -r
complete -c crisk -n "__fish_crisk_using_subcommand collect" -s o -l output -d 'Write the facts to FILE instead of stdout' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand collect" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand collect" -s h -l help -d 'Print help'
complete -c crisk -n "__fish_crisk_using_subcommand collect" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand evaluate" -s f -l format -d 'Output format; inferred from the --output name when omitted, else `table`' -r -f -a "json\t'The assessment, as RFC 8785 JSON'
table\t'A summary for people'
markdown\t'A summary in Markdown, for a job summary or a merge request note'
in-toto\t'An unsigned in-toto Statement v1 whose subject is the head commit'
sarif\t'SARIF 2.1.0, one result per reason that names paths'
codequality\t'GitLab Code Quality, one issue per reason and path'"
complete -c crisk -n "__fish_crisk_using_subcommand evaluate" -s o -l output -d 'Write to FILE instead of stdout' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand evaluate" -l fail-on -d 'Exit 1 when the tier is at or above TIER (emergency is above high)' -r -f -a "low\t''
standard\t''
high\t''
emergency\t''"
complete -c crisk -n "__fish_crisk_using_subcommand evaluate" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand evaluate" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c crisk -n "__fish_crisk_using_subcommand evaluate" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l github -d 'Read the change from this GitHub repository (OWNER/REPO); requires --pr' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l pr -d 'The pull request number' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l gitlab -d 'Read the change from this GitLab project (GROUP/PROJECT); requires --mr' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l mr -d 'The merge request IID' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l gitlab-url -d 'The GitLab API base URL (https only)' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l max-pages -d 'Maximum pages per list endpoint (1..=1000)' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l base -d 'The target branch commit the map and policy are read from (default: the forge change\'s base)' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l head -d 'The change\'s head (default: the forge change\'s head, else HEAD)' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l checkout -d 'The local clone' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l declarations -d 'Declarations from a YAML file instead of the change\'s description' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l signal -d 'A signal file: sarif=PATH, privacy-datamap=PATH or acc=PATH (repeatable)' -r
complete -c crisk -n "__fish_crisk_using_subcommand assess" -s o -l output -d 'Write the facts to FILE instead of stdout' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand assess" -s f -l format -d 'Output format; inferred from the --output name when omitted, else `table`' -r -f -a "json\t'The assessment, as RFC 8785 JSON'
table\t'A summary for people'
markdown\t'A summary in Markdown, for a job summary or a merge request note'
in-toto\t'An unsigned in-toto Statement v1 whose subject is the head commit'
sarif\t'SARIF 2.1.0, one result per reason that names paths'
codequality\t'GitLab Code Quality, one issue per reason and path'"
complete -c crisk -n "__fish_crisk_using_subcommand assess" -l fail-on -d 'Exit 1 when the tier is at or above TIER (emergency is above high)' -r -f -a "low\t''
standard\t''
high\t''
emergency\t''"
complete -c crisk -n "__fish_crisk_using_subcommand assess" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand assess" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c crisk -n "__fish_crisk_using_subcommand assess" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand render" -s f -l format -d 'Output format; inferred from the --output name when omitted, else `table`' -r -f -a "json\t'The assessment, as RFC 8785 JSON'
table\t'A summary for people'
markdown\t'A summary in Markdown, for a job summary or a merge request note'
in-toto\t'An unsigned in-toto Statement v1 whose subject is the head commit'
sarif\t'SARIF 2.1.0, one result per reason that names paths'
codequality\t'GitLab Code Quality, one issue per reason and path'"
complete -c crisk -n "__fish_crisk_using_subcommand render" -s o -l output -d 'Write to FILE instead of stdout' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand render" -l fail-on -d 'Exit 1 when the tier is at or above TIER (emergency is above high)' -r -f -a "low\t''
standard\t''
high\t''
emergency\t''"
complete -c crisk -n "__fish_crisk_using_subcommand render" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand render" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c crisk -n "__fish_crisk_using_subcommand render" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand validate" -l kind -d 'Which document it is; detected when omitted' -r -f -a "facts\t''
assessment\t''
statement\t''
map\t''
policy\t''
requirements\t''
population\t''"
complete -c crisk -n "__fish_crisk_using_subcommand validate" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand validate" -s h -l help -d 'Print help'
complete -c crisk -n "__fish_crisk_using_subcommand validate" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand compile" -l to -d 'What to produce' -r -f -a "acc\t'agent-change-control\'s `tiers` and `missing_tier` policy section (YAML)'
requirements\t'The tool-neutral requirements document (JSON)'
prose\t'The policy as Markdown, for people and auditors'"
complete -c crisk -n "__fish_crisk_using_subcommand compile" -l policy -d 'The policy file; without it, the policy is read from --base' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand compile" -l map -d 'The map file, with --policy' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand compile" -l base -d 'Read .change-risk/ at this revision of --checkout (default HEAD)' -r
complete -c crisk -n "__fish_crisk_using_subcommand compile" -l checkout -r -F
complete -c crisk -n "__fish_crisk_using_subcommand compile" -s o -l output -d 'Write to FILE instead of stdout' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand compile" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand compile" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c crisk -n "__fish_crisk_using_subcommand compile" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand population" -l since -d 'First day of the window, inclusive (YYYY-MM-DD, UTC)' -r
complete -c crisk -n "__fish_crisk_using_subcommand population" -l until -d 'Last day of the window, inclusive (YYYY-MM-DD, UTC)' -r
complete -c crisk -n "__fish_crisk_using_subcommand population" -l policy -d 'The policy file; without it, the policy is read from --base of --checkout' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand population" -l base -r
complete -c crisk -n "__fish_crisk_using_subcommand population" -l checkout -r -F
complete -c crisk -n "__fish_crisk_using_subcommand population" -l gitlab-url -r
complete -c crisk -n "__fish_crisk_using_subcommand population" -l max-pages -r
complete -c crisk -n "__fish_crisk_using_subcommand population" -s o -l output -d 'Write to FILE instead of stdout' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand population" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand population" -s h -l help -d 'Print help'
complete -c crisk -n "__fish_crisk_using_subcommand population" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand completions" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand completions" -s h -l help -d 'Print help'
complete -c crisk -n "__fish_crisk_using_subcommand completions" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand manpage" -l out-dir -d 'Write one page per command into DIR' -r -F
complete -c crisk -n "__fish_crisk_using_subcommand manpage" -s q -l quiet -d 'Suppress status lines on stderr'
complete -c crisk -n "__fish_crisk_using_subcommand manpage" -s h -l help -d 'Print help'
complete -c crisk -n "__fish_crisk_using_subcommand manpage" -s V -l version -d 'Print version'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "collect" -d 'Collect the facts of one change: paths from git, map and policy from the base revision, declarations and label events from the forge, labels from signal files'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "evaluate" -d 'Evaluate a facts document into an assessment, offline'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "assess" -d 'Collect and evaluate one change'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "render" -d 'Render an assessment (or a Statement) in another format, after validating it'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "validate" -d 'Validate a facts document, assessment, Statement, map, policy, requirements document or emergency population'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "compile" -d 'Compile the tier policy into acc\'s tier section, a requirements document or prose'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "population" -d 'List the emergency population: merged changes carrying the emergency label in a window'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "completions" -d 'Generate shell completions'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "manpage" -d 'Generate man pages'
complete -c crisk -n "__fish_crisk_using_subcommand help; and not __fish_seen_subcommand_from collect evaluate assess render validate compile population completions manpage help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
