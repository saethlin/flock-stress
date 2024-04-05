## File locking stress tester

```
touch lock
cargo r & cargo r
```

If that crashes, something about your filesystem's flock implementation is broken.

If you ctrl+c or otherwise kill one of the stress-tester processes, it may release the lock while the test file still exists, causing the other tester to crash. That does not indicate a flock problem.
