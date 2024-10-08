-- This file contains tests for the rendering engine.
-- Each test will start as a line beginning with "-- Test: <pines>", followed
-- by the expected outcome.
--
-- I have chose to use a .sql file as a test like this, because I will enjoy syntax highlighting
-- and autocompletion in my IDE. I've implemented some code that will report test failures as if
-- they were actual rust tests using assert!() macros.
--
-- Test: humans | s: id name
SELECT id, name
FROM humans
LIMIT 10;

-- Test: humans id=1
SELECT *
FROM humans
WHERE id = 1
LIMIT 10;

-- Test: humans | s: call()
SELECT call()
FROM humans
LIMIT 10;

-- Test: humans | s: FUNCTION(id name FUNCTION2(id name))
SELECT FUNCTION(id, name, FUNCTION2(id, name))
FROM humans
LIMIT 10;

-- Literal values
-- Test: humans | s: "one million" 1_000_000
SELECT "one million", 1000000
FROM humans
LIMIT 10;

-- Conditions
-- Test: humans | s: 1 != FUNCTION(id) "2000" = 2_000
SELECT 1 != FUNCTION(id), "2000" = 2000
FROM humans
LIMIT 10;

-- Filters
-- Test: humans | where: id=1 name="Karl"
SELECT *
FROM humans
WHERE id = 1 AND name = "Karl"
LIMIT 10;

-- Test: humans | l: 2
SELECT *
FROM humans
LIMIT 2;

-- Test: humans | l: 10 20
SELECT *
FROM humans
LIMIT 10, 20;

-- Test: humans | o: id = 2
SELECT *
FROM humans
ORDER BY id = 2 DESC
LIMIT 10;

-- Test: humans | g: id 2 "test"=4
SELECT id, 2, "test" = 4, *
FROM humans
GROUP BY id, 2, "test" = 4
LIMIT 10;

-- selects: after a group: (which adds an implicit select *) removes the implicit select *
-- Test: humans | g: name | s: count(1)
SELECT name, count(1)
FROM humans
GROUP BY name
LIMIT 10;

-- Test: humans | g: name | o: count(1)+
SELECT name, *
FROM humans
GROUP BY name
ORDER BY count(1)
LIMIT 10;
