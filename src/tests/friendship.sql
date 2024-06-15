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


-- Tests below
-- Test: people | j: preferences people.id = preferences.personId
SELECT preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | j: preferences people.id = personId
SELECT preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | j: preferences id = preferences.personId
SELECT preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | j: preferences id = personId
SELECT preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Join on multiple conditions
-- Test: people | j: preferences id=personId name="Silvanus"
SELECT preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId AND people.name = "Silvanus"
LIMIT 10;

-- Auto-join
-- Test: friendMap | j: friendshipProperties
SELECT friendshipProperties.*
FROM friendshipProperties
LEFT JOIN friendMap ON friendMap.friendA = friendshipProperties.friendA AND friendMap.friendB = friendshipProperties.friendB
LIMIT 10;

-- Test: friendMap | friendshipProperties
SELECT friendshipProperties.*
FROM friendshipProperties
LEFT JOIN friendMap ON friendMap.friendA = friendshipProperties.friendA AND friendMap.friendB = friendshipProperties.friendB
LIMIT 10;

-- Test: people | s: id id? id!?
SELECT id, id IS NULL, id IS NOT NULL
FROM people
LIMIT 10;

-- Test: people | j: preferences | where: id=1 personId=3
SELECT preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
WHERE preferences.id = 1 AND preferences.personId = 3
LIMIT 10;

-- Test: friendMap | friendshipProperties tag="test"
SELECT friendshipProperties.*
FROM friendshipProperties
LEFT JOIN friendMap ON friendMap.friendA = friendshipProperties.friendA AND friendMap.friendB = friendshipProperties.friendB
WHERE friendshipProperties.tag = "test"
LIMIT 10;

-- Test: people | o: id- name+ dateOfBirth
SELECT *
FROM people
ORDER BY id DESC, name, dateOfBirth DESC
LIMIT 10;

-- Test: people | preferences | g: id 2 "test"=4
SELECT preferences.id, 2, "test" = 4, preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
GROUP BY preferences.id, 2, "test" = 4
LIMIT 10;

-- Test: people | s: id | preferences
SELECT people.id, preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | s: id | g: name
SELECT id, name
FROM people
GROUP BY name
LIMIT 10;

-- Test: people | s: id | g: name | preferences
SELECT people.id, people.name, preferences.*
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
GROUP BY people.name
LIMIT 10;

-- Test: people | u: id
SELECT name, dateOfBirth, placeOfBirth
FROM people
LIMIT 10;

-- Test: people | u: id name
SELECT dateOfBirth, placeOfBirth
FROM people
LIMIT 10;
