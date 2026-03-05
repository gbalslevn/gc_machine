# Integration tests 

Testing all public functions work together. As someone who would use our API. Unit tests, tests of individual modules, are tested in the src folder. 

Run ```cargo test``` to test

# Code coverage
Use tarpaulin to get code coverage of unit tests:
```cargo tarpaulin --exclude-files "tests/*" --out html```