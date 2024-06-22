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

Using `stdin-to-query`
======================

I use this tool with vim, here's how. Let's say I have this pine I want to run:

```
users | userPreferences
```

1. With the cursor on that it, I enter visual line mode (Esc, V), this selects the entire line.
2. I hit my magic key-bind: `ctr+p` or `ctr+l`.

```text
" This is how I set it up.
" This is ctr+p, it prints results in a table.
:vmap <C-P><C-P> :'<,'>! /Users/fabianbadoi/projects/personal/rusty-pine/scripts/stdin-to-query 'db.hostname.com' <CR><CR><Esc>k$
" This is ctr+l, it prints results as objects.
:vmap <C-L><C-L> :'<,'>! /Users/fabianbadoi/projects/personal/rusty-pine/scripts/stdin-to-query 'db.hostname.com' '\G'<CR><CR><Esc>k$
```

3. I then get output like this.

```text
users | userPreferences   # <-- I keep editing this line, then hit the keybinding
=======================
users | userPreferences   # <-- I have records of all the pines I ran, in case I want to go back to it
-----------------------
SELECT userPreferences.*  # <-- Sometimes it's useful to see the query
FROM userPreferences
LEFT JOIN users ON users.id = userPreferences.userId
LIMIT 10;
< results here>
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
