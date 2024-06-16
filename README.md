All of this is out of date

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
3. Create a context

```bash
./target/release/pine create-context \
  --host <hostname> \
  --port <port> \
  --default-database <db name> \
  --username <username> \
  <name your context>
```

4. Use/enable the context

```bash
./target/release/pine use-context <your context name>
```

5. Run the `analyze` command so the tools knows learns the db structure

```bash
./target/release/pine analyze
```

You will be asked to pick which tables you want analyzed.

6. Translate your first pine

```bash
./target/release/pine translate "users email='spam@office.com'"
```

Mission Statement
-----------------

* you type less with rusty-pine
    - writing pine is way easier than writing SQL
* rusty-pine figures things out for you
    - if rusty-pine can do something, it will
* exploration focused
    - write less, edit less, jump around less with rusty-pine

[Pine]: https://github.com/ahmadnazir/pine
