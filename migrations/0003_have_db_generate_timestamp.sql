-- Migration number: 0003 	 2025-03-30T12:34:34.113Z
ALTER TABLE Attempts DROP COLUMN forty_five_timestamp;

ALTER TABLE Attempts ADD COLUMN forty_five_timestamp NUMBER NOT NULL;