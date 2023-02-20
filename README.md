# Akvy - simple HTTP api stress-test.

### Use on macOS

    ./akvy -u http://localhost:8080 -r 100

    http://localhost:8080/ | 100

    ^C        // Ctrl + C for stop

    Elapsed:             3.50s
    Requests:            350
    Errors:              0
    Percent of errors:   0.00%
    Response time:
    - Min:               0ms
    - Max:               11ms
    - Average:           1ms

### Compile for other OS

#### [Rust](https://www.rust-lang.org) must be installed.

        
    cd ./path/to/project

[]()

    cargo build --release

##### The binary file will be located in the directory
    
    ../target/release

