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
- [ ] Shorthand (s: for s:select, f: for from)
- [ ] select: *
- [ ] other comparisons
- [ ] Join statements
- [ ] Auto join
- [ ] Compact form:
    `users 3 | settings` instead of `from: users | where: id = 3 | join: settings | select: id`
- [ ] Support for limit
- [ ] Complex values
- [ ] Functions on a column
- [ ] Group on a column
- [ ] Order by a column
- [ ] Meta function show create
- [ ] Meta function show neighbors
- [ ] Support null checks
- [ ] Support multiple filters (OR)
- [ ] Updates
- [ ] Deletes
- [ ] Unselect specific columns

