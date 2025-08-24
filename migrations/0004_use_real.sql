-- Migration number: 0004 	 2025-03-30T13:16:14.654Z

ALTER TABLE Attempts DROP COLUMN forty_five_value;
ALTER TABLE Attempts DROP COLUMN forty_five_difference;

ALTER TABLE Attempts ADD COLUMN forty_five_value REAL;
ALTER TABLE Attempts ADD COLUMN forty_five_difference REAL;