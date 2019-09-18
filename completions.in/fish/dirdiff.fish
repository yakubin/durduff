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

complete -c {exe} -s q -l brief --description "Report only when directories differ."
complete -c {exe} -s p -l progress --description "Show progress bar."
complete -c {exe} -l color -xa "never always auto" --description "Print output in color."
complete -c {exe} -l percent --description "Utf-8 percent-encode paths."
complete -c {exe} -s b -l block-size -xa "" --description "Read files in blocks of <block-size> bytes."
complete -c {exe} -s h -l help --description "Print help information and exit."
complete -c {exe} -l version --description "Print version information and exit."

# complete exactly two directories
complete -c {exe} -xa ""
complete -c {exe} -n num_of_args_wo_opts_is_at_most_1 -xa '(__fish_complete_directories)'
