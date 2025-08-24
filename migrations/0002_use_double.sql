-- Migration number: 0002 	 2025-03-30T11:47:24.108Z

ALTER TABLE Attempts DROP COLUMN forty_five_value;
ALTER TABLE Attempts DROP COLUMN forty_five_difference;

ALTER TABLE Attempts ADD COLUMN forty_five_value DOUBLE;
ALTER TABLE Attempts ADD COLUMN forty_five_difference DOUBLE;