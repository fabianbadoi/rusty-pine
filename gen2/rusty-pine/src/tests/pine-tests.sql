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

-- TODO scalars in select
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
