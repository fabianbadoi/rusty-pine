-- This file contains tests for the rendering engine.
-- Each test will start as a line beginning with "-- Test: <pines>", followed
-- by the expected outcome.
--
-- I have chose to use a .sql file as a test like this, because I will enjoy syntax highlighting
-- and autocompletion in my IDE. I've implemented some code that will report test failures as if
-- they were actual rust tests using assert!() macros.

-- Test: humans | s: id name
SELECT id, name
FROM humans
LIMIT 10