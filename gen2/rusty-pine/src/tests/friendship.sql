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
SELECT *
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | j: preferences people.id = personId
SELECT *
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | j: preferences id = preferences.personId
SELECT *
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;

-- Test: people | j: preferences id = personId
SELECT *
FROM preferences
LEFT JOIN people ON people.id = preferences.personId
LIMIT 10;
