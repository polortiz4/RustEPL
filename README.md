# RustEPL
Tool for determining optimal Fantasy EPL teams given user-defined metrics
Pablo Ortiz <pablo..ortiz@duke.edu>
Finds Optimal EPL Fantasy Team

USAGE:
    rust_epl.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                     Prints help information
        --overwrite-pulled-team    True if you want to build your current squad manually instead of pulling, team would
                                   have to be hardcoded
    -p, --password                 True if fantasy password is to be provided manually, false if it's to be decoded from
                                   hardcoded encrypted password
    -V, --version                  Prints version information
    -v                             Sets verbosity

OPTIONS:
        --bench-point-value <bench_point_value>    Cost of a bench point [default: 5]
        --free-transfers <free_transfers>          Number of free transfers [default: 1]
    -g, --gameweek <gameweek>                      last week's gameweek number
        --min-player-metric <min_metric>           Minimum acceptable player metric
    -n, --top-n-players <top_n_players>
            Number of players to search in, that is the top n players in terms of metric [default: 20]

        --transfer-cost <transfer_cost>            Cost per transfer [default: 4]
    -u, --user-id <user_id>                        user-id from fantasy server to evaluate [default: 3521386]
