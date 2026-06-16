# Integration tests 

Testing all public functions work together. Unit tests, tests of individual modules, are tested in the src folder. 

Run ```cargo test``` to test

# Code coverage
Use tarpaulin to get code coverage of unit tests:
```cargo tarpaulin --exclude-files "tests/*" --out html```

# To test with AWS
**1**: Install AWS CLI https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html. 

**2** Compile the project as binary and upload to AWS/EC2 using e.g. scp. On EC2 run `cargo start` and ensure it has the correct network capabilities. 

**3** Run the AWS test in integration which runs another *Peer* locally ensure *Peer* is symmetrically setup as in `main.rs`, with the same `CircuitBuild`. 
