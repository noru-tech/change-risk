_crisk() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="crisk"
                ;;
            crisk,assess)
                cmd="crisk__subcmd__assess"
                ;;
            crisk,collect)
                cmd="crisk__subcmd__collect"
                ;;
            crisk,compile)
                cmd="crisk__subcmd__compile"
                ;;
            crisk,completions)
                cmd="crisk__subcmd__completions"
                ;;
            crisk,evaluate)
                cmd="crisk__subcmd__evaluate"
                ;;
            crisk,help)
                cmd="crisk__subcmd__help"
                ;;
            crisk,manpage)
                cmd="crisk__subcmd__manpage"
                ;;
            crisk,population)
                cmd="crisk__subcmd__population"
                ;;
            crisk,render)
                cmd="crisk__subcmd__render"
                ;;
            crisk,validate)
                cmd="crisk__subcmd__validate"
                ;;
            crisk__subcmd__help,assess)
                cmd="crisk__subcmd__help__subcmd__assess"
                ;;
            crisk__subcmd__help,collect)
                cmd="crisk__subcmd__help__subcmd__collect"
                ;;
            crisk__subcmd__help,compile)
                cmd="crisk__subcmd__help__subcmd__compile"
                ;;
            crisk__subcmd__help,completions)
                cmd="crisk__subcmd__help__subcmd__completions"
                ;;
            crisk__subcmd__help,evaluate)
                cmd="crisk__subcmd__help__subcmd__evaluate"
                ;;
            crisk__subcmd__help,help)
                cmd="crisk__subcmd__help__subcmd__help"
                ;;
            crisk__subcmd__help,manpage)
                cmd="crisk__subcmd__help__subcmd__manpage"
                ;;
            crisk__subcmd__help,population)
                cmd="crisk__subcmd__help__subcmd__population"
                ;;
            crisk__subcmd__help,render)
                cmd="crisk__subcmd__help__subcmd__render"
                ;;
            crisk__subcmd__help,validate)
                cmd="crisk__subcmd__help__subcmd__validate"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        crisk)
            opts="-q -h -V --quiet --help --version collect evaluate assess render validate compile population completions manpage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__assess)
            opts="-o -f -q -h -V --github --pr --gitlab --mr --gitlab-url --max-pages --base --head --checkout --declarations --signal --output --format --fail-on --quiet --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --github)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitlab)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitlab-url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-pages)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --base)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --head)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --checkout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --declarations)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --signal)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --format)
                    COMPREPLY=($(compgen -W "json table markdown in-toto sarif codequality" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "json table markdown in-toto sarif codequality" -- "${cur}"))
                    return 0
                    ;;
                --fail-on)
                    COMPREPLY=($(compgen -W "low standard high emergency" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__collect)
            opts="-o -q -h -V --github --pr --gitlab --mr --gitlab-url --max-pages --base --head --checkout --declarations --signal --output --quiet --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --github)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitlab)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitlab-url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-pages)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --base)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --head)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --checkout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --declarations)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --signal)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__compile)
            opts="-o -q -h -V --to --policy --map --base --checkout --output --quiet --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --to)
                    COMPREPLY=($(compgen -W "acc requirements prose" -- "${cur}"))
                    return 0
                    ;;
                --policy)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --map)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --base)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --checkout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__completions)
            opts="-q -h -V --quiet --help --version bash elvish fish powershell zsh"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__evaluate)
            opts="-f -o -q -h -V --format --output --fail-on --quiet --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "json table markdown in-toto sarif codequality" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "json table markdown in-toto sarif codequality" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --fail-on)
                    COMPREPLY=($(compgen -W "low standard high emergency" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help)
            opts="collect evaluate assess render validate compile population completions manpage help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__assess)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__collect)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__compile)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__completions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__evaluate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__manpage)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__population)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__render)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__help__subcmd__validate)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__manpage)
            opts="-q -h -V --out-dir --quiet --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --out-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__population)
            opts="-o -q -h -V --since --until --policy --base --checkout --gitlab-url --max-pages --output --quiet --help --version github gitlab"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --since)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --until)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --policy)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --base)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --checkout)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --gitlab-url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-pages)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__render)
            opts="-f -o -q -h -V --format --output --fail-on --quiet --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --format)
                    COMPREPLY=($(compgen -W "json table markdown in-toto sarif codequality" -- "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -W "json table markdown in-toto sarif codequality" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --fail-on)
                    COMPREPLY=($(compgen -W "low standard high emergency" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        crisk__subcmd__validate)
            opts="-q -h -V --kind --quiet --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --kind)
                    COMPREPLY=($(compgen -W "facts assessment statement map policy requirements population" -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _crisk -o nosort -o bashdefault -o default crisk
else
    complete -F _crisk -o bashdefault -o default crisk
fi
