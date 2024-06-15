-- These are the tests I wrote for the V1 version of this tool.
create table `people` (
    `id`           int auto_increment,
    `name`         varchar(256) null,
    `dateOfBirth`  date         not null,
    `placeOfBirth` varchar(256) not null,
    primary key (`id`),
)
    charset = utf8mb4;

create table `friendMap` (
    `friendA`  int                                                                               not null,
    `friendB`  int                                                                               not null,
    `strength` enum ('acquaintances', 'friends', 'bffs', 'two kittens sleeping in the same box') not null,
    primary key (`friendA`, `friendB`),
    constraint `friendMap_friendA_fk`
        foreign key (`friendA`) references `people` (`id`),
    constraint friendMap_friendB_fk
        foreign key (`friendB`) references `people` (`id`)
)
    charset = utf8mb4;

create table `friendshipLog` (
    `id`       int auto_increment,
    `friendA`  int  not null,
    `friendB`  int  not null,
    `metadata` text null,
    primary key (`id`),
    constraint `friendshipLog_friendA_fk`
        foreign key (`friendA`) references `people` (`id`),
    constraint friendshipLog_friendB_fk
        foreign key (`friendB`) references `people` (`id`)
)
    charset = utf8mb4;

create table `friendshipProperties` (
    `id`      int auto_increment,
    `friendA` int           not null,
    `friendB` int           not null,
    `tag`     varchar(64)   not null,
    `value`   varchar(1024) null,
    primary key (`id`),
    constraint friendshipProperties_fk
        foreign key (`friendA`, `friendB`) references `friendMap` (`friendA`, `friendB`)
)
    charset = utf8mb4;

create table `preferences` (
    `id`       int auto_increment,
    `personId` int          not null,
    `value`    varchar(256) not null,
    primary key (`id`),
    constraint `preferences_people_fk`
        foreign key (`personId`) references `people` (`id`)
)
    charset = utf8mb4;

create table `preferenceHistory` (
    `id`           int          not null,
    `preferenceId` int          not null,
    `oldValue`     varchar(256) null,
    primary key (`id`),
    constraint `preferenceHistory_preferenceId_fk`
        foreign key (`preferenceId`) references `preferences` (`id`)
)
    charset = utf8mb4;

-- Test: people | s: name | preferences
SELECT people.name, preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | s: id isBlue
SELECT id, isBlue
FROM people
LIMIT 10;

-- Test: people | preferences
SELECT preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | preferences | friendshipLog
SELECT friendshipLog.*
FROM friendshipLog
LEFT JOIN preferences ON preferences.personId = friendshipLog.friendA
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | w: id > 3 id < 3 id = 3 id != 3 id <= 3 id >= 3
SELECT *
FROM people
WHERE id > 3 AND id < 3 AND id = 3 AND id != 3 AND id <= 3 AND id >= 3
LIMIT 10;

-- Test: people | s: count(id) name
SELECT count(id), name
FROM people
LIMIT 10;

-- Test: people | s: count(id) name | g: name
SELECT count(id), name
FROM people
GROUP BY name
LIMIT 10;

-- Test: people | g: name | s: count(id)
SELECT name, count(id)
FROM people
GROUP BY name
LIMIT 10;

-- Test: people | g: count(id)
SELECT count(id), *
FROM people
GROUP BY count(id)
LIMIT 10;

-- Test: people | w: id > count(id)
SELECT *
FROM people
WHERE id > count(id)
LIMIT 10;

-- Test: people | friendshipLog |
/*
Foreign keys to:
  friendshipLog.friendA using .id
  friendshipLog.friendB using .id
*/--;

-- Test: people | s: count(1)
SELECT count(1)
FROM people
LIMIT 10;

