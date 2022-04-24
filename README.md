Rusty Pine
==========

It turns this:
```
users | select: id name | where: id = 3
```
Into this:
```sql
SELECT id, name
FROM users
WHERE id = 3
```

... and I plan on making it more powerful.

This project was inspired by Ahmad Nazir's [Pine], which I used a lot before writing this project.


Setup
=====

1. You'll need to install cargo.
2. Run `cargo build --release`
3. Create `~/.config/rusty-pine/config.json`:
```
{
    "user":"root",
    "password":"development",
    "host":"localhost",
    "port":3306
}
```
4. Run the analyze command from `target/release/analyze`, run this again after DB changes
5. Run `target/release/main "users email='spam@office.com'"


Logging
-------
Run with `RUST\_LOG=rusty\_pine\_lib=info` to enable logging.


Mission Statement
-----------------

* you type less with rusty-pine
    - writing pine is way easyer than writing SQL
* rusty-pine figures things out for you
    - if rusty-pine can do something, it will
* exploration focused
    - write less, edit less, jump around less with rusty-pine



TODO:
-----
- [x] Shorthand (s: for s:select, f: for from)
- [x] select: *
- [x] Compact form:
    `users 3 | settings` instead of `from: users | where: id = 3 | join: settings | select: id`
- [x] Implement Error on error
- [x] Join statements
- [x] Compound expression join statements
- [x] Support for limit
- [x] Implement live SQL analisys
- [x] Move complexity from SmartRenderer to Builder
- [x] Usable binaries
- [x] Add logging
- [x] Run clippy on code
- [x] Display available joins on join fail
- [x] All outputted errors must be SQL commnets
- [x] Order by a column
- [x] Support null checks
- [x] Add some integration test
- [x] Unselect specific columns
- [x] other comparisons
- [x] Functions on a column
- [x] Group on a column
- [x] Meta function show neighbors
- [x] Run without connection
- [x] Meta function show columns
- [ ] How to use
- [ ] Examples
- [ ] Read from stdin
- [ ] Complex values (`parentId = .id`?)
- [ ] Support multiple filters (OR)
- [ ] Updates
- [ ] Deletes


[Pine]: https://github.com/ahmadnazir/pine
