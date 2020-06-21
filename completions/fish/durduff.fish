function print_args_wo_opts
    commandline -poc \
        | awk 'NR==1 {next} /^(-b|--block-size|--color|--progress)$/{printf "%s ", $0; next} {print}' \
        | awk '/^-/{next} {print}'
end

function count_args_wo_opts
    count (print_args_wo_opts)
end

function num_of_args_wo_opts_is_at_most_1
    test (count_args_wo_opts) -le 1
end

complete -c durduff -s q -l brief --description "Report only when directories differ."
complete -c durduff -l progress -xa "never always auto" --description "Print progress reports."
complete -c durduff -l color -xa "never always auto" --description "Print output in color."
complete -c durduff -s 0 -l null -xa "" --description "Print raw NUL-separated paths."
complete -c durduff -s b -l block-size -xa "" --description "Read files in blocks of <block-size> bytes."
complete -c durduff -s h -l help --description "Print help information and exit."
complete -c durduff -l version --description "Print version information and exit."

# complete exactly two directories
complete -c durduff -xa ""
complete -c durduff -n num_of_args_wo_opts_is_at_most_1 -xa '(__fish_complete_directories)'
