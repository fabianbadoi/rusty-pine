Rusty Pine
==========

It turns this:
```
from: users | select id, name | where: id = 3
```
Into this:
```sql
SELECT id, name
FROM users
WHERE id = 3
```

... and I plan on making it more powerful.


Logging
-------
Run with `RUST_LOG=rusty_pine_lib=info` to enable logging.


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
- [ ] Display available joins on join fail
- [ ] Unselect specific columns
- [ ] All outputted errors must be SQL commnets
- [ ] other comparisons
- [ ] Complex values (`parentId = .id`?)
- [ ] Auto deep join
- [ ] Functions on a column
- [ ] Group on a column
- [ ] Order by a column
- [ ] Run without connection
- [ ] Meta function show create
- [ ] Meta function show neighbors
- [ ] Support null checks
- [ ] Support multiple filters (OR)
- [ ] Updates
- [ ] Deletes

