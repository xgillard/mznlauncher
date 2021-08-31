# MZNLAUNCH

This tool compiles a launcher (in rust) which parses instance files, translates
them into an input which is acceptable by the minizinc models and then spawns
`minizinc` to solve the problem in parallel (and let it run for a given max 
amount of time only ==> it makes sure to kill all subprocesses once the timeout
has elapsed).