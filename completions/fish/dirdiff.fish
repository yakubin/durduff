function print_args_wo_opts
    commandline -poc \
        | awk 'NR==1 {next} /^(-b|--block-size|--color)$/{printf "%s ", $0; next} {print}' \
        | awk '/^-/{next} {print}'
end

function count_args_wo_opts
    count (print_args_wo_opts)
end

function num_of_args_wo_opts_is_at_most_1
    test (count_args_wo_opts) -le 1
end

complete -c dirdiff -s q -l brief --description "Report only when directories differ."
complete -c dirdiff -s p -l progress --description "Show progress bar."
complete -c dirdiff -l color -xa "never always auto" --description "Print output in color."
complete -c dirdiff -l percent --description "Utf-8 percent-encode paths."
complete -c dirdiff -s b -l block-size -xa "" --description "Read files in blocks of <block-size> bytes."
complete -c dirdiff -s h -l help --description "Print help information and exit."
complete -c dirdiff -l version --description "Print version information and exit."

# complete exactly two directories
complete -c dirdiff -xa ""
complete -c dirdiff -n num_of_args_wo_opts_is_at_most_1 -xa '(__fish_complete_directories)'
